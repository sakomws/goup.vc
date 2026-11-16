//! Templates and data structures for the alliance site.
//!
//! The home page displays an overview of the alliance including alliance statistics,
//! upcoming events (both in-person and virtual), and recently added groups.

use askama::Template;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use ocg_core::types::{
    alliance::AllianceFull,
    event::{EventKind, EventSummary},
    group::GroupSummary,
    pagination::{NavigationLinks, Pagination, ToRawQuery},
    partner_integration::PartnerIntegration,
    site::SiteSettings,
};
use ocg_core::validation::{MAX_LEN_M, MAX_PAGINATION_LIMIT, trimmed_non_empty_opt};
use ocg_templates_common::PageId;
use ocg_templates_common::auth::User;
use ocg_templates_common::filters;
use ocg_templates_common::helpers::{self, user_initials};
use ocg_templates_dashboard::dashboard::{self, alliance::analytics::AllianceDashboardStats};

/// Link preview description for alliance pages.
pub const PREVIEW_DESCRIPTION: &str = "Open Alliance Groups, where Open Source alliances thrive.";
/// Link preview description for alliance brand pages.
pub const BRAND_PREVIEW_DESCRIPTION: &str = "Brand assets and identity links for this alliance.";

// Pages and sections templates.

/// Template for the alliance page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "alliance/page.html")]
pub struct Page {
    /// Configured public base URL.
    pub base_url: String,
    /// Alliance information.
    pub alliance: AllianceFull,
    /// Identifier for the current page.
    #[allow(dead_code)]
    pub page_id: PageId,
    /// Current request path.
    pub path: String,
    /// List of groups recently added to the alliance.
    pub recently_added_groups: Vec<GroupCard>,
    /// Global site settings.
    pub site_settings: SiteSettings,
    /// Aggregated statistics about groups, members, events, and attendees.
    pub stats: Stats,
    /// List of upcoming in-person events across all alliance groups.
    pub upcoming_in_person_events: Vec<EventCard>,
    /// List of upcoming virtual events across all alliance groups.
    pub upcoming_virtual_events: Vec<EventCard>,
    /// Authenticated user information.
    pub user: User,
}

impl Page {
    /// Returns the canonical public URL for the alliance page.
    pub fn canonical_url(&self) -> String {
        helpers::absolute_url(&self.base_url, &format!("/{}", self.alliance.name))
    }

    /// Returns the Open Graph image URL for the alliance page.
    pub fn open_graph_image_url(&self) -> Option<String> {
        self.alliance
            .og_image_url
            .as_deref()
            .map(|image_url| helpers::open_graph_image_url(&self.base_url, image_url))
    }

    /// Returns the preview title for the alliance page.
    pub fn preview_title(&self) -> String {
        format!("{} alliance", self.alliance.display_name)
    }
}

/// Template for the alliance brand page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "alliance/brand.html")]
pub struct BrandPage {
    /// Configured public base URL.
    pub base_url: String,
    /// Alliance information.
    pub alliance: AllianceFull,
    /// Identifier for the current page.
    #[allow(dead_code)]
    pub page_id: PageId,
    /// Current request path.
    pub path: String,
    /// Global site settings.
    pub site_settings: SiteSettings,
    /// Authenticated user information.
    pub user: User,
}

/// Template for the public alliance integrations page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "alliance/integrations.html")]
pub struct IntegrationsPage {
    /// Configured public base URL.
    pub base_url: String,
    /// Alliance information.
    pub alliance: AllianceFull,
    /// Enabled partner integrations configured by alliance administrators.
    pub integrations: Vec<PartnerIntegration>,
    /// Identifier for the current page.
    #[allow(dead_code)]
    pub page_id: PageId,
    /// Current request path.
    pub path: String,
    /// Global site settings.
    pub site_settings: SiteSettings,
    /// Authenticated user information.
    pub user: User,
}

/// Template for the alliance members page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "alliance/members.html")]
pub struct MembersPage {
    /// Configured public base URL.
    pub base_url: String,
    /// Alliance information.
    pub alliance: AllianceFull,
    /// Members across alliance groups.
    pub members: Vec<AllianceMember>,
    /// Pagination links.
    pub navigation_links: NavigationLinks,
    /// Number of results per page.
    pub offset: Option<usize>,
    /// Identifier for the current page.
    #[allow(dead_code)]
    pub page_id: PageId,
    /// Current request path.
    pub path: String,
    /// Search query.
    pub query: Option<String>,
    /// Global site settings.
    pub site_settings: SiteSettings,
    /// Total matching members.
    pub total: usize,
    /// Authenticated user information.
    pub user: User,
}

/// Template for the public alliance report page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "alliance/report.html")]
pub struct ReportPage {
    /// Configured public base URL.
    pub base_url: String,
    /// Alliance information.
    pub alliance: AllianceFull,
    /// Identifier for the current page.
    #[allow(dead_code)]
    pub page_id: PageId,
    /// Current request path.
    pub path: String,
    /// Global site settings.
    pub site_settings: SiteSettings,
    /// Alliance dashboard statistics.
    pub stats: AllianceDashboardStats,
    /// Authenticated user information.
    pub user: User,
}

impl MembersPage {
    /// Returns the canonical public URL for the alliance members page.
    pub fn canonical_url(&self) -> String {
        helpers::absolute_url(&self.base_url, &format!("/{}/members", self.alliance.name))
    }

    /// Returns the Open Graph image URL for the alliance members page.
    pub fn open_graph_image_url(&self) -> Option<String> {
        self.alliance
            .og_image_url
            .as_deref()
            .map(|image_url| helpers::open_graph_image_url(&self.base_url, image_url))
    }

    /// Returns the preview title for the alliance members page.
    pub fn preview_title(&self) -> String {
        format!("{} members", self.alliance.display_name)
    }

    /// Returns the preview description for the alliance members page.
    pub fn preview_description(&self) -> String {
        format!(
            "Browse member cards across {} groups.",
            self.alliance.display_name
        )
    }
}

impl ReportPage {
    /// Returns the canonical public URL for the alliance report page.
    pub fn canonical_url(&self) -> String {
        helpers::absolute_url(&self.base_url, &format!("/{}/reports", self.alliance.name))
    }

    /// Returns the Open Graph image URL for the alliance report page.
    pub fn open_graph_image_url(&self) -> Option<String> {
        self.alliance
            .og_image_url
            .as_deref()
            .map(|image_url| helpers::open_graph_image_url(&self.base_url, image_url))
    }

    /// Returns the preview title for the alliance report page.
    pub fn preview_title(&self) -> String {
        format!("{} report", self.alliance.display_name)
    }

    /// Returns the preview description for the alliance report page.
    pub fn preview_description(&self) -> String {
        format!(
            "Public growth and activity report for {}.",
            self.alliance.display_name
        )
    }
}

pub use ocg_core::shared::AllianceMember;

/// Filter parameters for alliance member directory pagination.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, garde::Validate)]
pub struct AllianceMembersFilters {
    /// Number of results per page.
    #[serde(default = "dashboard::default_limit")]
    #[garde(range(max = MAX_PAGINATION_LIMIT))]
    pub limit: Option<usize>,
    /// Pagination offset for results.
    #[serde(default = "dashboard::default_offset")]
    #[garde(skip)]
    pub offset: Option<usize>,
    /// Text search query.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_M))]
    pub query: Option<String>,
}

ocg_core::impl_pagination_and_raw_query!(AllianceMembersFilters, limit, offset);

/// Paginated alliance members response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceMembersOutput {
    /// Members across alliance groups.
    pub members: Vec<AllianceMember>,
    /// Total matching members.
    pub total: usize,
}

/// Private member fields returned only for an authorized alliance CSV export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceMemberExport {
    pub name: Option<String>,
    pub username: String,
    pub email: String,
    pub phone_country_code: Option<String>,
    pub phone_number: Option<String>,
    pub group_names: Vec<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub linkedin_url: Option<String>,
    pub github_url: Option<String>,
    pub website_url: Option<String>,
}

impl BrandPage {
    /// Returns the canonical public URL for the alliance brand page.
    pub fn canonical_url(&self) -> String {
        helpers::absolute_url(&self.base_url, &format!("/{}/brand", self.alliance.name))
    }

    /// Returns the Open Graph image URL for the alliance brand page.
    pub fn open_graph_image_url(&self) -> Option<String> {
        self.alliance
            .og_image_url
            .as_deref()
            .or(Some(self.alliance.logo_url.as_str()))
            .map(|image_url| helpers::open_graph_image_url(&self.base_url, image_url))
    }

    /// Returns the preview title for the alliance brand page.
    pub fn preview_title(&self) -> String {
        format!("{} brand", self.alliance.display_name)
    }
}

impl IntegrationsPage {
    /// Returns the canonical public URL for the integrations page.
    pub fn canonical_url(&self) -> String {
        helpers::absolute_url(
            &self.base_url,
            &format!("/{}/integrations", self.alliance.name),
        )
    }

    /// Returns the Open Graph image URL for the integrations page.
    pub fn open_graph_image_url(&self) -> Option<String> {
        self.alliance
            .og_image_url
            .as_deref()
            .or(Some(self.alliance.logo_url.as_str()))
            .map(|image_url| helpers::open_graph_image_url(&self.base_url, image_url))
    }

    /// Returns the preview title for the integrations page.
    pub fn preview_title(&self) -> String {
        format!("{} integrations", self.alliance.display_name)
    }
}

/// Event card template for home page display.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "common/event_card_small.html")]
pub struct EventCard {
    /// Event data
    pub event: EventSummary,
}

/// Group card template for home page display.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "alliance/group_card.html")]
pub struct GroupCard {
    /// Group data
    pub group: GroupSummary,
}

/// Alliance statistics for the home page.
#[derive(Debug, Clone, Default, Template, Serialize, Deserialize)]
#[template(path = "alliance/stats.html")]
pub struct Stats {
    /// Total number of groups in the alliance.
    pub groups: i64,
    /// Total number of members across all groups.
    pub groups_members: i64,
    /// Total number of events hosted by all groups.
    pub events: i64,
    /// Total number of attendees across all events.
    pub events_attendees: i64,
}
