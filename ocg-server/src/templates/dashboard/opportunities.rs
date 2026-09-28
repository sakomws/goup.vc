//! Opportunity organizer dashboard templates.

use askama::Template;
use axum_messages::{Level, Message};
use serde::{Deserialize, Serialize};

use crate::{
    templates::{PageId, auth::User, filters, helpers::user_initials},
    types::{
        opportunities::{
            DashboardOpportunityFilters, OpportunitiesOutput, OpportunitySavedSearch,
            OpportunitySummary,
        },
        pagination::NavigationLinks,
        site::SiteSettings,
    },
};

#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/opportunities.html")]
pub(crate) struct Page {
    pub messages: Vec<Message>,
    #[allow(dead_code)]
    pub page_id: PageId,
    pub path: String,
    pub site_settings: SiteSettings,
    pub user: User,
    pub filters: DashboardOpportunityFilters,
    pub opportunities: Vec<OpportunitySummary>,
    pub total: usize,
    pub navigation_links: NavigationLinks,
    pub saved_searches: Vec<OpportunitySavedSearch>,
    pub preview_name: Option<String>,
    pub preview: Option<OpportunitiesOutput>,
}
