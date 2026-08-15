//! HTTP handlers for the public landscape page.

use std::{cmp::Reverse, time::Duration};

use askama::Template;
use axum::{
    extract::{RawQuery, State},
    response::{Html, IntoResponse},
};
use cached::cached;
use chrono::{DateTime, Utc};
use garde::Validate;
use reqwest::Url;
use serde::Deserialize;
use tracing::{debug, instrument};

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::{error::HandlerError, extend_public_shared_cache_headers},
    router::serde_qs_config,
    templates::{
        PageId,
        auth::User,
        site::landscape::{
            self, GitHubLeaderboard, GitHubProjectLeaderboardEntry, GitHubRepositoryMetrics,
        },
    },
    types::{landscape::LandscapeFilters, pagination::NavigationLinks},
    validation::MAX_PAGINATION_LIMIT,
};

const LANDSCAPE_URL: &str = "/landscape";
const GITHUB_PROJECT_KIND: &str = "github_project";
const GITHUB_LEADERBOARD_LIMIT: usize = 25;
const GITHUB_LEADERBOARD_DISPLAY_LIMIT: usize = 10;
const GITHUB_SORT_STARS: &str = "stars";
const GITHUB_SORT_FORKS: &str = "forks";
const GITHUB_SORT_UPDATED: &str = "updated";

/// Render the public landscape listing page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    let filters = parse_filters(raw_query.as_deref().unwrap_or_default())?;
    let github_filters = github_leaderboard_filters(&filters);
    let logo_filters = logo_strip_filters(&filters);
    let (output, logo_output, github_output, site_settings) = tokio::try_join!(
        db.search_landscape_entries(&filters),
        db.search_landscape_entries(&logo_filters),
        db.search_landscape_entries(&github_filters),
        db.get_site_settings()
    )?;
    // In local development the landscape table is usually empty, which leaves
    // the whole page blank. Fall back to mock data so the marquee, leaderboard,
    // and directory are all reviewable while developing. Mocks only ever engage
    // in debug builds, so production is unaffected regardless of what the
    // database returns.
    #[cfg(debug_assertions)]
    let use_mocks = output.entries.is_empty() && logo_output.entries.is_empty();
    #[cfg(not(debug_assertions))]
    let use_mocks = false;

    #[cfg(debug_assertions)]
    let (logo_entries, entries, total) = if use_mocks {
        let matching = dev_mocks::mock_entries(&filters);
        let total = matching.len();
        // Paginate like the database would, so the pager stays testable.
        let entries = matching
            .into_iter()
            .skip(filters.offset.unwrap_or(0))
            .take(filters.limit.unwrap_or(MAX_PAGINATION_LIMIT))
            .collect();
        // The marquee always shows the unfiltered set.
        (
            dev_mocks::mock_entries(&LandscapeFilters::default()),
            entries,
            total,
        )
    } else {
        (logo_output.entries, output.entries, output.total)
    };
    #[cfg(not(debug_assertions))]
    let (logo_entries, entries, total) = (logo_output.entries, output.entries, output.total);

    let navigation_links =
        NavigationLinks::from_filters(&filters, total, LANDSCAPE_URL, LANDSCAPE_URL)?;

    let github_leaderboard = if should_show_github_leaderboard(&filters) {
        let sort = github_leaderboard_sort(&filters);
        match mock_github_leaderboard(use_mocks, sort) {
            Some(leaderboard) => leaderboard,
            None => load_github_leaderboard(&github_output.entries, sort).await,
        }
    } else {
        GitHubLeaderboard::default()
    };

    let template = landscape::Page {
        page_id: PageId::SiteLandscape,
        path: LANDSCAPE_URL.to_string(),
        site_settings,
        user: User::from_session(auth_session).await?,
        filters,
        github_leaderboard,
        logo_entries,
        entries,
        total,
        navigation_links,
    };

    Ok((
        extend_public_shared_cache_headers(&[])?,
        Html(template.render()?),
    ))
}

/// Returns a mock leaderboard when local development is running on mock data.
/// Mocked runs must not call the live GitHub API: the mock repositories either
/// do not exist upstream or would burn rate limit, and a failed lookup would
/// hide the section entirely.
#[cfg(debug_assertions)]
fn mock_github_leaderboard(use_mocks: bool, sort: &'static str) -> Option<GitHubLeaderboard> {
    use_mocks.then(|| dev_mocks::mock_github_leaderboard(sort))
}

/// Release builds never substitute mock leaderboard data.
#[cfg(not(debug_assertions))]
fn mock_github_leaderboard(_use_mocks: bool, _sort: &'static str) -> Option<GitHubLeaderboard> {
    None
}

fn parse_filters(raw_query: &str) -> Result<LandscapeFilters, HandlerError> {
    let filters: LandscapeFilters = if raw_query.is_empty() {
        LandscapeFilters::default()
    } else {
        serde_qs_config().deserialize_str(raw_query)?
    };
    filters.validate()?;
    Ok(filters)
}

fn github_leaderboard_filters(filters: &LandscapeFilters) -> LandscapeFilters {
    LandscapeFilters {
        kind: Some(GITHUB_PROJECT_KIND.to_string()),
        limit: Some(GITHUB_LEADERBOARD_LIMIT),
        offset: Some(0),
        ..filters.clone()
    }
}

fn logo_strip_filters(filters: &LandscapeFilters) -> LandscapeFilters {
    LandscapeFilters {
        limit: Some(MAX_PAGINATION_LIMIT),
        offset: Some(0),
        ..filters.clone()
    }
}

fn should_show_github_leaderboard(filters: &LandscapeFilters) -> bool {
    filters.kind.as_deref().is_none_or(|kind| kind == GITHUB_PROJECT_KIND)
}

fn github_leaderboard_sort(filters: &LandscapeFilters) -> &'static str {
    match filters.github_sort.as_deref() {
        Some(GITHUB_SORT_FORKS) => GITHUB_SORT_FORKS,
        Some(GITHUB_SORT_UPDATED) => GITHUB_SORT_UPDATED,
        _ => GITHUB_SORT_STARS,
    }
}

async fn load_github_leaderboard(
    entries: &[crate::types::landscape::LandscapeEntry],
    sort: &'static str,
) -> GitHubLeaderboard {
    let mut leaderboard = Vec::new();
    let mut attempted_count = 0;
    let mut unavailable_count = 0;

    for entry in entries {
        let Some(github_url) = entry.github_url.as_deref() else {
            continue;
        };
        let Some((owner, repo)) = parse_github_repository_url(github_url) else {
            continue;
        };
        attempted_count += 1;
        let Some(metrics) = fetch_github_repository_metrics(owner.clone(), repo.clone()).await
        else {
            unavailable_count += 1;
            continue;
        };

        leaderboard.push(GitHubProjectLeaderboardEntry {
            entry: entry.clone(),
            repository: format!("{owner}/{repo}"),
            score: leaderboard_score(&metrics, sort),
            share_pct: 0,
            metrics,
        });
    }

    sort_github_leaderboard(&mut leaderboard, sort);
    leaderboard.truncate(GITHUB_LEADERBOARD_DISPLAY_LIMIT);
    apply_share_percentages(&mut leaderboard);
    GitHubLeaderboard {
        entries: leaderboard,
        attempted_count,
        unavailable_count,
        sort: sort.to_string(),
    }
}

/// Scores each entry as a percentage of the highest score in the leaderboard,
/// used to size the grading bar shown next to every project. Entries always
/// keep a small visible minimum so low-scoring projects still render a bar.
fn apply_share_percentages(leaderboard: &mut [GitHubProjectLeaderboardEntry]) {
    const MIN_SHARE_PCT: u8 = 6;

    let top_score = leaderboard.iter().map(|project| project.score).max().unwrap_or(0);
    for project in &mut *leaderboard {
        project.share_pct = if top_score > 0 {
            let pct = project.score.saturating_mul(100) / top_score;
            u8::try_from(pct).unwrap_or(100).clamp(MIN_SHARE_PCT, 100)
        } else {
            MIN_SHARE_PCT
        };
    }
}

fn leaderboard_score(metrics: &GitHubRepositoryMetrics, sort: &str) -> i64 {
    match sort {
        GITHUB_SORT_FORKS => metrics.forks_count,
        _ => metrics.stargazers_count,
    }
}

fn sort_github_leaderboard(leaderboard: &mut [GitHubProjectLeaderboardEntry], sort: &str) {
    match sort {
        GITHUB_SORT_FORKS => leaderboard.sort_by_key(|project| {
            (
                Reverse(project.metrics.forks_count),
                Reverse(project.metrics.stargazers_count),
                project.entry.name.to_lowercase(),
            )
        }),
        GITHUB_SORT_UPDATED => leaderboard.sort_by_key(|project| {
            (
                Reverse(project.metrics.updated_at),
                Reverse(project.metrics.stargazers_count),
                project.entry.name.to_lowercase(),
            )
        }),
        _ => leaderboard.sort_by_key(|project| {
            (
                Reverse(project.metrics.stargazers_count),
                Reverse(project.metrics.forks_count),
                project.entry.name.to_lowercase(),
            )
        }),
    }
}

fn parse_github_repository_url(url: &str) -> Option<(String, String)> {
    let url = Url::parse(url).ok()?;
    if url.host_str()? != "github.com" {
        return None;
    }

    let mut segments = url.path_segments()?;
    let owner = segments.next()?.trim();
    let repo = segments.next()?.trim().trim_end_matches(".git");
    if owner.is_empty() || repo.is_empty() {
        return None;
    }

    Some((owner.to_string(), repo.to_string()))
}

#[cached(ttl = 900)]
async fn fetch_github_repository_metrics(
    owner: String,
    repo: String,
) -> Option<GitHubRepositoryMetrics> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("GOUP Landscape/1.0")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let url = format!("https://api.github.com/repos/{owner}/{repo}");
    let response = client.get(url).send().await;
    let response = match response {
        Ok(response) => response,
        Err(error) => {
            debug!("failed to fetch GitHub repository metrics for {owner}/{repo}: {error}");
            return None;
        }
    };

    match response.error_for_status() {
        Ok(response) => response.json::<GitHubRepositoryResponse>().await.ok().map(Into::into),
        Err(error) => {
            debug!("GitHub repository metrics unavailable for {owner}/{repo}: {error}");
            None
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubRepositoryResponse {
    stargazers_count: i64,
    forks_count: i64,
    open_issues_count: i64,
    watchers_count: i64,
    subscribers_count: Option<i64>,
    updated_at: Option<DateTime<Utc>>,
    pushed_at: Option<DateTime<Utc>>,
}

impl From<GitHubRepositoryResponse> for GitHubRepositoryMetrics {
    fn from(value: GitHubRepositoryResponse) -> Self {
        Self {
            stargazers_count: value.stargazers_count,
            forks_count: value.forks_count,
            open_issues_count: value.open_issues_count,
            watchers_count: value.subscribers_count.unwrap_or(value.watchers_count),
            updated_at: value.updated_at,
            pushed_at: value.pushed_at,
        }
    }
}

/// Mock landscape data used only in local development, when the database has
/// no landscape entries to show. Compiled out of release builds.
#[cfg(debug_assertions)]
mod dev_mocks {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::{
        templates::site::landscape::{
            GitHubLeaderboard, GitHubProjectLeaderboardEntry, GitHubRepositoryMetrics,
        },
        types::landscape::{LandscapeAcceleratorProfile, LandscapeEntry, LandscapeFilters},
    };

    /// A mock landscape record, carrying the same fields production data does.
    struct MockEntry {
        name: &'static str,
        kind: &'static str,
        category: &'static str,
        summary: &'static str,
        tags: &'static [&'static str],
        /// Repository path, for entries backed by a GitHub repository.
        repository: Option<&'static str>,
        /// Mock repository metrics as (stars, forks, open issues, watchers).
        metrics: Option<(i64, i64, i64, i64)>,
    }

    /// Local-only landscape records covering every entry kind, so the marquee,
    /// GitHub leaderboard, and directory cards all have realistic data.
    const MOCK_ENTRIES: &[MockEntry] = &[
        MockEntry {
            name: "Zapcast",
            kind: "startup",
            category: "AutoTech",
            summary: "An automotive parts marketplace connecting drivers with spare parts sellers through instant quote requests.",
            tags: &["Marketplace", "Logistics"],
            repository: None,
            metrics: None,
        },
        MockEntry {
            name: "CanYouHack",
            kind: "startup",
            category: "Cybersecurity",
            summary: "Hosts capture-the-flag challenges that let users practice real offensive and defensive security skills in a safe environment.",
            tags: &["Security", "Education"],
            repository: None,
            metrics: None,
        },
        MockEntry {
            name: "Hands_on_ML_Azerbaijani",
            kind: "github_project",
            category: "ML",
            summary: "Azerbaijani notes, summaries, and learning materials based on the book Hands-on Machine Learning.",
            tags: &["Machine Learning", "Localization"],
            repository: Some("Lala2398/Hands_on_ML_Azerbaijani"),
            metrics: Some((62, 9, 0, 4)),
        },
        MockEntry {
            name: "OzunOyren",
            kind: "startup",
            category: "EdTech",
            summary: "An online learning platform focused on practical, competency-based education in technology, business, and professional skills.",
            tags: &["Education", "Careers"],
            repository: None,
            metrics: None,
        },
        MockEntry {
            name: "Bebras",
            kind: "partner_community",
            category: "EdTech",
            summary: "An international challenge on informatics and computational thinking that introduces school students to problem solving through short, engaging tasks.",
            tags: &["Community", "Students"],
            repository: None,
            metrics: None,
        },
        MockEntry {
            name: "CheckOwners",
            kind: "github_project",
            category: "AI",
            summary: "Derives CODEOWNERS from git history with confidence scoring, expertise decay detection, and bus factor analysis. Pure git, no LLMs.",
            tags: &["Developer Tools", "CI"],
            repository: Some("Turall/CheckOwners"),
            metrics: Some((31, 4, 2, 3)),
        },
        MockEntry {
            name: "Cuprice",
            kind: "startup",
            category: "AI",
            summary: "AI native pricing infrastructure for founders and digital service providers, helping teams build and continuously improve pricing strategies.",
            tags: &["Pricing", "SaaS"],
            repository: None,
            metrics: None,
        },
        MockEntry {
            name: "FlatWhite",
            kind: "startup",
            category: "FoodTech",
            summary: "Community-built coffee logging app for people who take their coffee seriously. Log and rate individual drinks and build a visual Coffee Passport.",
            tags: &["Consumer", "Mobile"],
            repository: None,
            metrics: None,
        },
        MockEntry {
            name: "Rastock AI",
            kind: "startup",
            category: "AI",
            summary: "Automatically generates SEO-optimized titles, keywords, and descriptions for content and uploads them in bulk to stock platforms.",
            tags: &["Automation", "SEO"],
            repository: None,
            metrics: None,
        },
        MockEntry {
            name: "Opa-python-client",
            kind: "github_project",
            category: "Developer Tools",
            summary: "A Python client for the Open Policy Agent REST API, covering policy management and document evaluation.",
            tags: &["Python", "Policy"],
            repository: Some("Turall/OPA-python-client"),
            metrics: Some((65, 14, 0, 3)),
        },
        MockEntry {
            name: "Roda Ledger",
            kind: "github_project",
            category: "FinTech",
            summary: "A double-entry ledger service with a typed API for building accounting and balance tracking into products.",
            tags: &["Ledger", "Accounting"],
            repository: Some("tislib/roda-ledger"),
            metrics: Some((16, 2, 1, 1)),
        },
        MockEntry {
            name: "cache-house",
            kind: "github_project",
            category: "Developer Tools",
            summary: "A caching layer with pluggable backends and a small, predictable API surface for Python services.",
            tags: &["Caching", "Python"],
            repository: Some("Turall/cache-house"),
            metrics: Some((16, 1, 0, 2)),
        },
        MockEntry {
            name: "fastapi-ldap",
            kind: "github_project",
            category: "Developer Tools",
            summary: "LDAP authentication integration for FastAPI applications, with session handling and role mapping.",
            tags: &["FastAPI", "Auth"],
            repository: Some("Turall/fastapi-ldap"),
            metrics: Some((15, 1, 0, 1)),
        },
        MockEntry {
            name: "goup.vc",
            kind: "github_project",
            category: "Community",
            summary: "The platform behind the GOUP alliance: events, groups, jobs, and this ecosystem landscape.",
            tags: &["Rust", "Community"],
            repository: Some("sakomws/goup.vc"),
            metrics: Some((13, 3, 1, 1)),
        },
        MockEntry {
            name: "DS Roadmap",
            kind: "github_project",
            category: "ML",
            summary: "A structured data science learning roadmap with curated resources for each stage of the path.",
            tags: &["Data Science", "Learning"],
            repository: Some("AzizNadirov/ds-roadmap"),
            metrics: Some((11, 1, 0, 1)),
        },
        MockEntry {
            name: "ParVu - Parquet Viewer",
            kind: "github_project",
            category: "Data",
            summary: "A desktop viewer for Parquet files with schema inspection and quick querying.",
            tags: &["Parquet", "Desktop"],
            repository: Some("AzizNadirov/ParVu"),
            metrics: Some((10, 2, 2, 1)),
        },
        MockEntry {
            name: "kubechronicle",
            kind: "github_project",
            category: "Infrastructure",
            summary: "Tracks and narrates Kubernetes cluster change history so teams can see what shifted and when.",
            tags: &["Kubernetes", "Observability"],
            repository: Some("Turall/kubechronicle"),
            metrics: Some((8, 1, 0, 0)),
        },
        MockEntry {
            name: "GOUP Accelerator",
            kind: "accelerator",
            category: "Program",
            summary: "A cohort-based program for alliance founders, pairing weekly build sessions with distribution and fundraising support.",
            tags: &["Cohort", "Founders"],
            repository: None,
            metrics: None,
        },
        MockEntry {
            name: "Open Technology",
            kind: "partner_community",
            category: "Community",
            summary: "A partner community running open source meetups, workshops, and contribution drives across the region.",
            tags: &["Open Source", "Meetups"],
            repository: None,
            metrics: None,
        },
        MockEntry {
            name: "Alliance Ventures",
            kind: "investor",
            category: "Pre-seed",
            summary: "An early-stage fund backing alliance founders at pre-seed, with a focus on developer tools and applied AI.",
            tags: &["Pre-seed", "Fund"],
            repository: None,
            metrics: None,
        },
        MockEntry {
            name: "Builders Podcast",
            kind: "podcast_lead",
            category: "Media",
            summary: "A podcast lead covering founder stories from the alliance, recorded live at community events.",
            tags: &["Podcast", "Stories"],
            repository: None,
            metrics: None,
        },
    ];

    /// Builds mock landscape entries, honouring the kind and free-text filters
    /// so the search controls remain testable against mock data.
    pub(super) fn mock_entries(filters: &LandscapeFilters) -> Vec<LandscapeEntry> {
        let now = Utc::now();
        let query = filters.query.as_deref().map(str::to_lowercase);
        let category = filters.category.as_deref().map(str::to_lowercase);

        MOCK_ENTRIES
            .iter()
            .filter(|mock| filters.kind.as_deref().is_none_or(|kind| kind == mock.kind))
            .filter(|mock| {
                category
                    .as_deref()
                    .is_none_or(|category| mock.category.to_lowercase().contains(category))
            })
            .filter(|mock| {
                query.as_deref().is_none_or(|query| {
                    mock.name.to_lowercase().contains(query)
                        || mock.summary.to_lowercase().contains(query)
                        || mock.category.to_lowercase().contains(query)
                        || mock.tags.iter().any(|tag| tag.to_lowercase().contains(query))
                })
            })
            .map(|mock| to_landscape_entry(mock, now))
            .collect()
    }

    /// Builds a mock GitHub leaderboard from the mock repositories, reusing the
    /// production ranking so sorting behaves identically to a live run.
    pub(super) fn mock_github_leaderboard(sort: &'static str) -> GitHubLeaderboard {
        let now = Utc::now();
        let mut leaderboard: Vec<GitHubProjectLeaderboardEntry> = MOCK_ENTRIES
            .iter()
            .enumerate()
            .filter_map(|(index, mock)| {
                let repository = mock.repository?;
                let (stargazers_count, forks_count, open_issues_count, watchers_count) =
                    mock.metrics?;
                // Stagger update times so the "recently updated" sort is
                // visibly different from the star and fork rankings.
                let age = Duration::days(i64::try_from(index).unwrap_or(0));
                let metrics = GitHubRepositoryMetrics {
                    stargazers_count,
                    forks_count,
                    open_issues_count,
                    watchers_count,
                    updated_at: Some(now - age),
                    pushed_at: Some(now - age),
                };

                Some(GitHubProjectLeaderboardEntry {
                    entry: to_landscape_entry(mock, now),
                    repository: repository.to_string(),
                    score: super::leaderboard_score(&metrics, sort),
                    share_pct: 0,
                    metrics,
                })
            })
            .collect();

        let attempted_count = leaderboard.len();
        super::sort_github_leaderboard(&mut leaderboard, sort);
        leaderboard.truncate(super::GITHUB_LEADERBOARD_DISPLAY_LIMIT);
        super::apply_share_percentages(&mut leaderboard);

        GitHubLeaderboard {
            entries: leaderboard,
            attempted_count,
            unavailable_count: 0,
            sort: sort.to_string(),
        }
    }

    /// Converts a mock record into the landscape entry the templates consume.
    fn to_landscape_entry(mock: &MockEntry, now: chrono::DateTime<Utc>) -> LandscapeEntry {
        let accelerator = (mock.kind == "accelerator").then(|| LandscapeAcceleratorProfile {
            application_url: Some("#".to_string()),
            curriculum_url: Some("#".to_string()),
            cohort_status: Some("open".to_string()),
            starts_on: Some("2026-09-01".to_string()),
            ends_on: Some("2026-12-04".to_string()),
            tracks: vec![
                "AI".to_string(),
                "Open Source".to_string(),
                "Revenue".to_string(),
            ],
            weekly_agenda: None,
            updated_at: now,
        });

        LandscapeEntry {
            landscape_entry_id: Uuid::new_v4(),
            alliance_id: Uuid::nil(),
            added_by_user_id: Uuid::nil(),
            name: mock.name.to_string(),
            slug: mock.name.to_lowercase().replace(' ', "-"),
            kind: mock.kind.to_string(),
            summary: mock.summary.to_string(),
            description: None,
            website_url: mock.repository.is_none().then(|| "#".to_string()),
            github_url: mock
                .repository
                .map(|repository| format!("https://github.com/{repository}")),
            logo_url: None,
            category: Some(mock.category.to_string()),
            stage: None,
            tags: mock.tags.iter().map(|tag| (*tag).to_string()).collect(),
            published: true,
            affiliations: Vec::new(),
            accelerator,
            created_at: now,
            updated_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_repository_urls() {
        assert_eq!(
            parse_github_repository_url("https://github.com/rust-lang/rust"),
            Some(("rust-lang".to_string(), "rust".to_string()))
        );
        assert_eq!(
            parse_github_repository_url("https://github.com/owner/repo.git"),
            Some(("owner".to_string(), "repo".to_string()))
        );
        assert_eq!(
            parse_github_repository_url("https://github.com/owner"),
            None
        );
        assert_eq!(
            parse_github_repository_url("https://gitlab.com/owner/repo"),
            None
        );
    }

    #[test]
    fn computes_leaderboard_share_percentages() {
        fn project(score: i64) -> GitHubProjectLeaderboardEntry {
            GitHubProjectLeaderboardEntry {
                entry: crate::types::landscape::LandscapeEntry {
                    landscape_entry_id: uuid::Uuid::nil(),
                    alliance_id: uuid::Uuid::nil(),
                    added_by_user_id: uuid::Uuid::nil(),
                    name: "Repo".to_string(),
                    slug: "repo".to_string(),
                    kind: GITHUB_PROJECT_KIND.to_string(),
                    summary: "Summary".to_string(),
                    description: None,
                    website_url: None,
                    github_url: None,
                    logo_url: None,
                    category: None,
                    stage: None,
                    tags: Vec::new(),
                    published: true,
                    affiliations: Vec::new(),
                    accelerator: None,
                    created_at: Utc::now(),
                    updated_at: None,
                },
                repository: "owner/repo".to_string(),
                score,
                share_pct: 0,
                metrics: GitHubRepositoryMetrics {
                    stargazers_count: score,
                    forks_count: 0,
                    open_issues_count: 0,
                    watchers_count: 0,
                    updated_at: None,
                    pushed_at: None,
                },
            }
        }

        let mut leaderboard = vec![project(100), project(50), project(1), project(0)];
        apply_share_percentages(&mut leaderboard);
        assert_eq!(leaderboard[0].share_pct, 100);
        assert_eq!(leaderboard[1].share_pct, 50);
        // Low and zero scores are floored so their bars stay visible.
        assert_eq!(leaderboard[2].share_pct, 6);
        assert_eq!(leaderboard[3].share_pct, 6);

        // An all-zero leaderboard must not divide by zero.
        let mut zeroed = vec![project(0), project(0)];
        apply_share_percentages(&mut zeroed);
        assert!(zeroed.iter().all(|project| project.share_pct == 6));
    }

    #[test]
    fn normalizes_github_leaderboard_sort() {
        assert_eq!(
            github_leaderboard_sort(&LandscapeFilters {
                github_sort: Some(GITHUB_SORT_FORKS.to_string()),
                ..Default::default()
            }),
            GITHUB_SORT_FORKS
        );
        assert_eq!(
            github_leaderboard_sort(&LandscapeFilters {
                github_sort: Some(GITHUB_SORT_UPDATED.to_string()),
                ..Default::default()
            }),
            GITHUB_SORT_UPDATED
        );
        assert_eq!(
            github_leaderboard_sort(&LandscapeFilters {
                github_sort: Some("unknown".to_string()),
                ..Default::default()
            }),
            GITHUB_SORT_STARS
        );
    }
}
