//! Scheduled discovery and publication of global jobs from user-approved sources.

use std::{collections::HashSet, sync::Arc, time::Duration};

use anyhow::Result;
use chrono::{Datelike, TimeZone, Timelike, Utc};
use garde::Validate;
use tokio::time::sleep;
use tokio_postgres::types::Json;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    config::YouComConfig,
    db::{PgDB, PgExecutor, jobs::DBJobs},
    integrations::{
        event_page::validate_candidate_url,
        job_page::JobPageClient,
        you_com::{SearchResult, YouComClient, source_search_domain},
    },
    types::jobs::JobInput,
};

/// Runs an authorized user-owned discovery request without blocking HTTP.
#[derive(Clone)]
pub(crate) struct ManualJobDiscovery {
    cfg: YouComConfig,
    db: Arc<PgDB>,
}

impl ManualJobDiscovery {
    pub(crate) fn new(cfg: YouComConfig, db: Arc<PgDB>) -> Self {
        Self { cfg, db }
    }
    pub(crate) fn enabled(&self) -> bool {
        self.cfg.enabled
    }
    pub(crate) fn spawn_user_run(&self, user_id: Uuid) {
        let cfg = self.cfg.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            if let Err(err) = run_user(cfg, db, user_id).await {
                error!(%err, %user_id, "manual You.com job discovery failed");
            }
        });
    }
}

/// Starts the daily job discovery worker using the shared You.com configuration.
pub(crate) fn start(
    cfg: YouComConfig,
    db: Arc<PgDB>,
    tasks: &TaskTracker,
    cancel: &CancellationToken,
) {
    if !cfg.enabled {
        return;
    }
    tasks.spawn({
        let cancel = cancel.clone();
        async move {
            let Ok(client) = YouComClient::new(&cfg) else {
                error!("could not start You.com job discovery");
                return;
            };
            let Ok(timezone) = cfg.schedule_timezone.parse() else {
                error!("invalid You.com job discovery timezone");
                return;
            };
            loop {
                tokio::select! {
                    () = sleep(delay_until_next_run(timezone, cfg.schedule_hour)) => {},
                    () = cancel.cancelled() => break,
                }
                if let Err(err) = ingest_enabled(&db, &client).await {
                    error!(%err, "scheduled You.com job discovery failed");
                }
            }
        }
    });
}

pub(crate) async fn run_user(cfg: YouComConfig, db: Arc<PgDB>, user_id: Uuid) -> Result<()> {
    anyhow::ensure!(cfg.enabled, "You.com job discovery is disabled");
    let enabled: bool = db.fetch_scalar_one(
        "select exists(select 1 from jobs_discovery_integration where user_id = $1 and enabled)",
        &[&user_id],
    ).await?;
    anyhow::ensure!(enabled, "job discovery is not enabled");
    ingest_users(&db, &YouComClient::new(&cfg)?, &[user_id]).await
}

async fn ingest_enabled(db: &PgDB, client: &YouComClient) -> Result<()> {
    let users: Vec<Uuid> = db.fetch_scalar_one(
        "select coalesce(array_agg(user_id), '{}'::uuid[]) from jobs_discovery_integration where enabled",
        &[],
    ).await?;
    ingest_users(db, client, &users).await
}

async fn ingest_users(db: &PgDB, client: &YouComClient, users: &[Uuid]) -> Result<()> {
    for user_id in users {
        db.execute(
            "insert into jobs_discovery_run (user_id, status) values ($1, 'running')",
            &[user_id],
        )
        .await?;
    }
    let outcome = async {
        let job_pages = JobPageClient::new()?;
        for user_id in users {
            let sources: Vec<String> = db.fetch_scalar_one(
                "select coalesce(array_agg(url), '{}'::text[]) from jobs_discovery_source
                 where user_id = $1 and enabled", &[user_id],
            ).await?;
            anyhow::ensure!(
                !sources.is_empty(),
                "no enabled job discovery sources are configured"
            );
            let mut discovered = 0;
            let mut created = 0;
            for source_url in sources {
                let mut seen = HashSet::new();
                let search_domain = source_search_domain(&source_url)?;
                let mut candidates: Vec<(String, JobInput)> = job_pages
                    .discover_greenhouse_jobs(&source_url)
                    .await?
                    .into_iter()
                    .map(|input| (input.apply_url.clone(), input))
                    .collect();
                for result in client
                    .search(&format!("jobs hiring careers site:{search_domain}"))
                    .await?
                {
                    if let Err(err) = validate_candidate_url(&result.url, &source_url).await {
                        info!(
                            %err,
                            candidate_url = %result.url,
                            source_url = %source_url,
                            "skipped unsafe job discovery candidate"
                        );
                        continue;
                    }
                    let input = match job_pages.fetch(&result.url, &source_url).await {
                        Ok(input) => input.or_else(|| parse_discovered_job(&result)),
                        Err(_) => parse_discovered_job(&result),
                    };
                    if let Some(input) = input {
                        candidates.push((result.url, input));
                    }
                }
                for (candidate_url, input) in candidates {
                    if !seen.insert(candidate_url.trim().to_lowercase()) {
                        continue;
                    }
                    let fingerprint = fingerprint(&input, &candidate_url);
                    let item: Option<Uuid> = db.fetch_scalar_opt(
                        "insert into jobs_discovery_item (
                            user_id, source_url, candidate_url, fingerprint, discovered_payload
                         ) values ($1, $2, $3, $4, $5) on conflict (user_id, fingerprint) do nothing
                         returning jobs_discovery_item_id",
                        &[
                            user_id,
                            &source_url,
                            &candidate_url,
                            &fingerprint,
                            &Json(&input),
                        ],
                    ).await?;
                    let Some(_) = item else { continue };
                    discovered += 1;
                    let job_id = db.add_job(*user_id, &input).await?;
                    db.update_job_published(*user_id, job_id, false).await?;
                    db.execute(
                        "update jobs_discovery_item set job_id = $1 where user_id = $2 and fingerprint = $3",
                        &[&job_id, user_id, &fingerprint],
                    ).await?;
                    created += 1;
                }
            }
            db.execute(
                "update jobs_discovery_run set completed_at = now(), status = 'succeeded',
                 discovered_count = $2, created_count = $3
                 where jobs_discovery_run_id = (select jobs_discovery_run_id from jobs_discovery_run
                 where user_id = $1 and status = 'running' order by started_at desc limit 1)",
                &[user_id, &discovered, &created],
            ).await?;
            info!(%user_id, discovered, created, "published discovered jobs");
        }
        Result::<()>::Ok(())
    }.await;
    if let Err(err) = &outcome {
        for user_id in users {
            db.execute(
                "update jobs_discovery_run set completed_at = now(), status = 'failed', error_message = $2
                 where jobs_discovery_run_id = (select jobs_discovery_run_id from jobs_discovery_run
                 where user_id = $1 and status = 'running' order by started_at desc limit 1)",
                &[user_id, &err.to_string()],
            ).await?;
        }
    }
    outcome
}

/// Accept only results with an explicit title, employer, description, and HTTPS application URL.
fn parse_discovered_job(result: &SearchResult) -> Option<JobInput> {
    let title = result.title.trim();
    let description = result
        .description
        .as_deref()
        .or_else(|| result.snippets.first().map(String::as_str))?
        .trim();
    let apply_url = reqwest::Url::parse(result.url.trim()).ok()?;
    if !matches!(apply_url.scheme(), "http" | "https") || title.is_empty() || description.is_empty()
    {
        return None;
    }
    // You.com result titles conventionally expose the employer after " at " or " - ".
    let (title, company_name) = title
        .split_once(" at ")
        .or_else(|| title.split_once(" - "))
        .filter(|(role, company)| !role.trim().is_empty() && !company.trim().is_empty())?;
    if is_generic_job_page(title) {
        return None;
    }
    let summary: String = description.chars().take(280).collect();
    let input = JobInput {
        title: title.trim().to_owned(),
        company_name: company_name.trim().to_owned(),
        summary,
        description: description.to_owned(),
        apply_url: apply_url.into(),
        location: None,
        remote: None,
        members_only: Some(false),
        tags: None,
    };
    input.validate().ok()?;
    Some(input)
}

/// Reject search, index, and landing-page titles that do not identify a role.
fn is_generic_job_page(role: &str) -> bool {
    const GENERIC_PREFIXES: &[&str] = &[
        "career opportunities",
        "job opportunities",
        "search results",
        "search result",
        "search jobs",
        "job search",
        "find jobs",
        "careers",
        "jobs",
    ];

    let normalized = role.trim().to_ascii_lowercase();
    GENERIC_PREFIXES.iter().any(|prefix| {
        normalized == *prefix
            || normalized.starts_with(&format!("{prefix} |"))
            || normalized.starts_with(&format!("{prefix}:"))
    })
}

fn fingerprint(input: &JobInput, apply_url: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(input.title.trim().to_lowercase());
    hash.update(b"\0");
    hash.update(input.company_name.trim().to_lowercase());
    hash.update(b"\0");
    hash.update(apply_url.trim().to_lowercase());
    hex::encode(hash.finalize())
}

fn delay_until_next_run(timezone: chrono_tz::Tz, hour: u8) -> Duration {
    let now = Utc::now().with_timezone(&timezone);
    let today = timezone
        .with_ymd_and_hms(now.year(), now.month(), now.day(), u32::from(hour), 0, 0)
        .single()
        .expect("configured schedule hour must be valid");
    let next = if now.hour() < u32::from(hour) {
        today
    } else {
        today + chrono::Duration::days(1)
    };
    (next.with_timezone(&Utc) - Utc::now()).to_std().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_incomplete_search_results() {
        assert!(
            parse_discovered_job(&SearchResult {
                title: "Engineer".into(),
                url: "https://x.test".into(),
                description: None,
                snippets: vec![],
            })
            .is_none()
        );
    }
    #[test]
    fn accepts_explicit_job_result() {
        let job = parse_discovered_job(&SearchResult {
            title: "Rust Engineer at Acme".into(),
            url: "https://x.test/jobs/1".into(),
            description: Some("Build reliable systems.".into()),
            snippets: vec![],
        })
        .unwrap();
        assert_eq!(job.company_name, "Acme");
    }

    #[test]
    fn accepts_snippet_when_description_is_missing() {
        let job = parse_discovered_job(&SearchResult {
            title: "Rust Engineer at Acme".into(),
            url: "https://x.test/jobs/1".into(),
            description: None,
            snippets: vec!["Build reliable systems.".into()],
        })
        .unwrap();
        assert_eq!(job.summary, "Build reliable systems.");
    }

    #[test]
    fn rejects_search_result_landing_pages() {
        let job = parse_discovered_job(&SearchResult {
            title: "Search Results | Find available job openings - BCG Careers".into(),
            url: "https://careers.bcg.com/search-results".into(),
            description: Some("Have questions about careers at BCG?".into()),
            snippets: vec![],
        });
        assert!(job.is_none());
    }
}
