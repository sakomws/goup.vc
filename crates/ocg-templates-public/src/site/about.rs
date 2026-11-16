//! Templates for the public about page.

use askama::Template;
use serde::{Deserialize, Serialize};

use ocg_core::types::site::SiteSettings;
use ocg_templates_common::PageId;
use ocg_templates_common::auth::User;
use ocg_templates_common::filters;
use ocg_templates_common::helpers::user_initials;

/// Public about page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "site/about/page.html")]
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
}
