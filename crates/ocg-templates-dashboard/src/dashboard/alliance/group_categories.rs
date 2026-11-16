//! Templates and types for managing group categories in the alliance dashboard.

use askama::Template;
use garde::Validate;
use serde::{Deserialize, Serialize};

use ocg_core::types::group::GroupCategory;
use ocg_core::validation::{MAX_LEN_ENTITY_NAME, trimmed_non_empty};

// Pages templates.

/// Group categories list page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/alliance/group_categories_list.html")]
pub struct ListPage {
    /// Whether the current user can manage taxonomy.
    pub can_manage_taxonomy: bool,
    /// Group categories available in the selected alliance.
    pub categories: Vec<GroupCategory>,
}

/// Group category add form template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/alliance/group_categories_add.html")]
pub struct AddPage {
    /// Whether the current user can manage taxonomy.
    pub can_manage_taxonomy: bool,
}

/// Group category update form template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/alliance/group_categories_update.html")]
pub struct UpdatePage {
    /// Whether the current user can manage taxonomy.
    pub can_manage_taxonomy: bool,
    /// Group category currently being edited.
    pub category: GroupCategory,
}

// Types.

/// Group category form payload used by create and update operations.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GroupCategoryInput {
    /// Group category name.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_ENTITY_NAME))]
    pub name: String,
}
