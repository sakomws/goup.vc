//! Templates and types for managing the group team in the dashboard.

use askama::Template;
use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::dashboard;
use ocg_core::types::{
    group::{GroupRole, GroupRoleSummary},
    pagination::{self, Pagination, ToRawQuery},
};
use ocg_core::validation::MAX_PAGINATION_LIMIT;
use ocg_templates_common::helpers::user_initials;

// Pages templates.

/// List team members page template for a group.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/group/team_list.html")]
pub struct ListPage {
    /// Whether the current user can update team membership and roles.
    pub can_manage_team: bool,
    /// Tooltip shown when the current user cannot update team membership.
    pub manage_team_disabled_message: Option<String>,
    /// List of team members in the group.
    pub members: Vec<GroupTeamMember>,
    /// Pagination navigation links.
    pub navigation_links: pagination::NavigationLinks,
    /// List of available team roles.
    pub roles: Vec<GroupRoleSummary>,
    /// Total number of team members.
    pub total: usize,
    /// Number of accepted members in the group team.
    pub total_accepted: usize,
    /// Number of accepted admins in the group team.
    pub total_admins_accepted: usize,

    /// Number of results per page.
    pub limit: Option<usize>,
    /// Pagination offset for results.
    pub offset: Option<usize>,
}

impl ListPage {
    /// Returns the tooltip text for disabled group team management controls.
    pub fn manage_team_disabled_message(&self) -> &str {
        self.manage_team_disabled_message.as_deref().unwrap_or("")
    }
}

// Types.

/// Filter parameters for group team pagination.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct GroupTeamFilters {
    /// Number of results per page.
    #[serde(default = "dashboard::default_limit")]
    #[garde(range(max = MAX_PAGINATION_LIMIT))]
    pub limit: Option<usize>,
    /// Pagination offset for results.
    #[serde(default = "dashboard::default_offset")]
    #[garde(skip)]
    pub offset: Option<usize>,
}

ocg_core::impl_pagination_and_raw_query!(GroupTeamFilters, limit, offset);

/// Group team member summary information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTeamMember {
    /// Whether the membership has been accepted.
    pub accepted: bool,
    /// Unique identifier for the user.
    pub user_id: Uuid,
    /// Username.
    pub username: String,

    /// Company the user represents.
    pub company: Option<String>,
    /// Full name.
    pub name: Option<String>,
    /// URL to user's avatar.
    pub photo_url: Option<String>,
    /// Team role.
    pub role: Option<GroupRole>,
    /// Title held by the user.
    pub title: Option<String>,
}

/// Paginated group team response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTeamOutput {
    /// List of team members in the group.
    pub members: Vec<GroupTeamMember>,
    /// Total number of team members.
    pub total: usize,
    /// Total number of accepted members.
    pub total_accepted: usize,
    /// Total number of accepted admins.
    pub total_admins_accepted: usize,
}
