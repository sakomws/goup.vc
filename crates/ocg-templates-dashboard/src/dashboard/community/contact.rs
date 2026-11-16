//! Templates for the contact section of the community dashboard.

use askama::Template;
use serde::{Deserialize, Serialize};

use ocg_core::types::{
    dashboard::community::contact::{
        CommunityContactFilterOptions, CommunityContactRecipientsSummary,
    },
    group::GroupRoleSummary,
};
use ocg_core::validation::MAX_CONTACT_FILTER_VALUES;

#[cfg(test)]
mod tests;

/// Group role identifiers in the order the role filter displays them.
const ROLES_ORDER: [&str; 4] = ["admin", "events-manager", "check-in-manager", "viewer"];

// Pages templates.

/// Contact page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/alliance/contact.html")]
pub struct Page {
    /// Whether the current user can send emails to group teams.
    pub can_send: bool,
    /// Default subject for the notification email.
    pub default_notification_subject: String,
    /// Filter options available in the community.
    pub filter_options: CommunityContactFilterOptions,
    /// Canonical query of the filters the initial summary was computed for.
    pub filters_key: String,
    /// Group team roles available in the role filter.
    pub roles: Vec<GroupRoleSummary>,
    /// Recipients matching the initial filters.
    pub summary: CommunityContactRecipientsSummary,
}

impl Page {
    /// Returns the group category filter options with their group counts.
    pub fn category_options(&self) -> Vec<MultiSelectOption> {
        self.filter_options
            .group_categories
            .iter()
            .map(|category| MultiSelectOption {
                name: format!("{} ({})", category.name, category.groups_count),
                value: category.group_category_id.to_string(),
            })
            .collect()
    }

    /// Returns the group filter options labelled by name, category and region,
    /// in the order the database returned them.
    pub fn group_options(&self) -> Vec<MultiSelectOption> {
        self.filter_options
            .groups
            .iter()
            .map(|group| MultiSelectOption {
                name: group.label.clone(),
                value: group.group_id.to_string(),
            })
            .collect()
    }

    /// Returns whether the community has active groups to filter by.
    pub fn has_groups(&self) -> bool {
        !self.filter_options.groups.is_empty()
    }

    /// Returns the groups filter legend, including the subgroup hint when needed.
    pub fn groups_legend(&self) -> String {
        let mut legend = format!("Select up to {MAX_CONTACT_FILTER_VALUES} groups.");
        if self.filter_options.has_subgroups {
            legend.push_str(" Selecting a group doesn't include its subgroups; select them too.");
        }
        legend
    }

    /// Returns whether the community has regions to filter by.
    pub fn has_regions(&self) -> bool {
        !self.filter_options.regions.is_empty()
    }

    /// Returns the region filter options with their group counts, followed by
    /// the option selecting groups without a region.
    pub fn region_options(&self) -> Vec<MultiSelectOption> {
        self.filter_options
            .regions
            .iter()
            .map(|region| MultiSelectOption {
                name: format!("{} ({})", region.name, region.groups_count),
                value: region.region_id.to_string(),
            })
            .chain(std::iter::once(MultiSelectOption {
                name: format!("No region ({})", self.filter_options.no_region_groups_count),
                value: "none".to_string(),
            }))
            .collect()
    }

    /// Returns the group role filter options in their fixed display order.
    pub fn role_options(&self) -> Vec<MultiSelectOption> {
        let mut roles: Vec<&GroupRoleSummary> = self.roles.iter().collect();
        roles.sort_by_key(|role| {
            ROLES_ORDER
                .iter()
                .position(|id| *id == role.group_role_id)
                .unwrap_or(ROLES_ORDER.len())
        });
        roles
            .into_iter()
            .map(|role| MultiSelectOption {
                name: role.display_name.clone(),
                value: role.group_role_id.clone(),
            })
            .collect()
    }
}

/// Recipients summary partial template.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/alliance/contact_recipients.html")]
pub struct RecipientsSummary {
    /// Canonical query of the filters the summary was computed for.
    pub filters_key: String,
    /// Recipients matching the filters.
    pub summary: CommunityContactRecipientsSummary,
}

// Types.

/// Option offered by a `<multi-select>` filter.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MultiSelectOption {
    /// Label shown for the option.
    pub name: String,
    /// Value submitted when the option is selected.
    pub value: String,
}
