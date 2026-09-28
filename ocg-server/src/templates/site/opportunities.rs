//! Public opportunity board templates.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::{
    templates::{PageId, auth::User, filters, helpers::user_initials},
    types::{
        opportunities::{OpportunityFilters, OpportunitySummary},
        pagination::NavigationLinks,
        site::SiteSettings,
    },
};

#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "site/opportunities/page.html")]
pub(crate) struct Page {
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
pub(crate) struct DetailsPage {
    #[allow(dead_code)]
    pub page_id: PageId,
    pub path: String,
    pub site_settings: SiteSettings,
    pub user: User,
    pub opportunity: OpportunitySummary,
}
