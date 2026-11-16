//! Templates and types for the public landscape page.

use askama::Template;
use serde::{Deserialize, Serialize};

use ocg_core::types::{
    landscape::{LandscapeEntry, LandscapeFilters},
    pagination::NavigationLinks,
    site::SiteSettings,
};
use ocg_templates_common::PageId;
use ocg_templates_common::auth::User;
use ocg_templates_common::filters;
use ocg_templates_common::helpers::user_initials;

/// Public landscape listing page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "site/landscape/page.html")]
pub struct Page {
    /// Identifier for the current page.
    #[allow(dead_code)]
    pub page_id: PageId,
    /// Current URL path.
    pub path: String,
    /// Global site settings.
    pub site_settings: SiteSettings,
    /// Authenticated user information.
    pub user: User,
    /// Search filters.
    pub filters: LandscapeFilters,
    /// GitHub projects ranked by live repository metrics.
    pub github_leaderboard: GitHubLeaderboard,
    /// Landscape entries shown in the top logo strip.
    pub logo_entries: Vec<LandscapeEntry>,
    /// Matching landscape entries.
    pub entries: Vec<LandscapeEntry>,
    /// Total matching entries.
    pub total: usize,
    /// Pagination links.
    pub navigation_links: NavigationLinks,
}

/// GitHub leaderboard display data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubLeaderboard {
    /// Ranked GitHub project entries.
    pub entries: Vec<GitHubProjectLeaderboardEntry>,
    /// Number of landscape entries considered for the leaderboard.
    pub attempted_count: usize,
    /// Number of repositories skipped because GitHub metrics were unavailable.
    pub unavailable_count: usize,
    /// Active leaderboard sort option.
    pub sort: String,
}

/// GitHub project with live repository metrics for leaderboard display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubProjectLeaderboardEntry {
    /// Landscape entry backing the leaderboard row.
    pub entry: LandscapeEntry,
    /// Repository path in owner/name form.
    pub repository: String,
    /// Primary leaderboard score.
    pub score: i64,
    /// Score as a percentage of the top-ranked entry, used for the grading bar.
    pub share_pct: u8,
    /// Live repository metrics.
    pub metrics: GitHubRepositoryMetrics,
}

/// Public GitHub repository metrics displayed on the landscape leaderboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepositoryMetrics {
    /// GitHub star count.
    pub stargazers_count: i64,
    /// GitHub fork count.
    pub forks_count: i64,
    /// GitHub open issue count.
    pub open_issues_count: i64,
    /// GitHub watcher count.
    pub watchers_count: i64,
    /// GitHub repository update time.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    /// GitHub repository push time.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub pushed_at: Option<chrono::DateTime<chrono::Utc>>,
}
