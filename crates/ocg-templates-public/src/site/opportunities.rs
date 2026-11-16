//! Public opportunity board templates.

use askama::Template;
use serde::{Deserialize, Serialize};

use ocg_core::types::{
    opportunities::{OpportunityFilters, OpportunitySummary},
    pagination::NavigationLinks,
    site::SiteSettings,
};
use ocg_templates_common::PageId;
use ocg_templates_common::auth::User;
use ocg_templates_common::filters;
use ocg_templates_common::helpers::user_initials;

#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "site/opportunities/page.html")]
pub struct Page {
    #[allow(dead_code)]
    pub page_id: PageId,
    pub path: String,
    pub site_settings: SiteSettings,
    pub user: User,
    pub filters: OpportunityFilters,
    pub csv_url: String,
    pub opportunities: Vec<OpportunitySummary>,
    pub total: usize,
    pub navigation_links: NavigationLinks,
}

#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "site/opportunities/details.html")]
pub struct DetailsPage {
    #[allow(dead_code)]
    pub page_id: PageId,
    pub path: String,
    pub site_settings: SiteSettings,
    pub user: User,
    pub opportunity: OpportunitySummary,
}
