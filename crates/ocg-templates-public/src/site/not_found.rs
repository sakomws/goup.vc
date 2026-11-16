//! Templates for the global site not found page.

use askama::Template;
use serde::{Deserialize, Serialize};

use ocg_core::types::site::SiteSettings;
use ocg_templates_common::PageId;
use ocg_templates_common::auth::User;
use ocg_templates_common::filters;
use ocg_templates_common::helpers::user_initials;

// Pages templates.

/// Template for rendering the not found page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "site/not_found/page.html")]
pub struct Page {
    /// Identifier for the current page.
    #[allow(dead_code)]
    pub page_id: PageId,
    /// Stable path used by the shared base and header templates.
    pub path: String,
    /// Global site settings.
    pub site_settings: SiteSettings,
    /// Authenticated user information.
    pub user: User,
}
