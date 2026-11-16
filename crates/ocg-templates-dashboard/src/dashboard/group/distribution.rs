//! Group distribution dashboard template.

use askama::Template;
use serde::{Deserialize, Serialize};

use ocg_core::types::distribution::DistributionDashboard;

#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/group/distribution.html")]
pub struct Page {
    pub can_manage: bool,
    pub dashboard: DistributionDashboard,
}
