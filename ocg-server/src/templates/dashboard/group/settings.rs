//! Templates for the group dashboard settings page.

use askama::Template;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::types::group::{GroupCategory, GroupFull, GroupParentOption, GroupRegion};

// Pages templates.

/// Update page template for group settings.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/group/settings_update.html")]
pub(crate) struct UpdatePage {
    /// Whether the current user can manage settings.
    pub can_manage_settings: bool,
    /// List of available group categories.
    pub categories: Vec<GroupCategory>,
    /// Group information.
    pub group: GroupFull,
    /// Groups that can be selected as a parent chapter.
    pub parent_group_options: Vec<GroupParentOption>,
    /// Whether payments are globally enabled.
    pub payments_enabled: bool,
    /// List of available regions.
    pub regions: Vec<GroupRegion>,
}

impl UpdatePage {
    /// Returns true when the option matches the group's current parent.
    pub(crate) fn is_selected_parent_group(&self, group_id: &Uuid) -> bool {
        self.group.parent_group_id.as_ref() == Some(group_id)
    }
}

// Types.

/// Group update form data (alias for the Group type from alliance dashboard).
pub(crate) use crate::templates::dashboard::alliance::groups::Group as GroupUpdate;
