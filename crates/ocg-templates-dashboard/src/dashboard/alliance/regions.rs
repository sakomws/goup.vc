//! Templates and types for managing regions in the alliance dashboard.

use askama::Template;
use garde::Validate;
use serde::{Deserialize, Serialize};

use ocg_core::types::group::GroupRegion;
use ocg_core::validation::{MAX_LEN_ENTITY_NAME, trimmed_non_empty};

// Pages templates.

/// Regions list page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/alliance/regions_list.html")]
pub struct ListPage {
    /// Whether the current user can manage taxonomy.
    pub can_manage_taxonomy: bool,
    /// Regions available in the selected alliance.
    pub regions: Vec<GroupRegion>,
}

/// Region add form template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/alliance/regions_add.html")]
pub struct AddPage {
    /// Whether the current user can manage taxonomy.
    pub can_manage_taxonomy: bool,
}

/// Region update form template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/alliance/regions_update.html")]
pub struct UpdatePage {
    /// Whether the current user can manage taxonomy.
    pub can_manage_taxonomy: bool,
    /// Region currently being edited.
    pub region: GroupRegion,
}

// Types.

/// Region form payload used by create and update operations.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegionInput {
    /// Region name.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_ENTITY_NAME))]
    pub name: String,
}
