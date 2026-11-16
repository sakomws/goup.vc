//! Templates for the user dashboard affiliations tab.

use askama::Template;
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ocg_templates_common::filters;

pub use ocg_core::types::user::{AFFILIATION_ROLES, role_label};

/// List page showing the current user's affiliations.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/user/affiliations_list.html")]
pub struct ListPage {
    /// Affiliations the user has declared.
    pub affiliations: Vec<UserAffiliation>,
    /// Published landscape entries available for selection, ordered by kind and name.
    pub entry_options: Vec<LandscapeEntryOption>,
}

impl ListPage {
    /// Groups the entry options by kind, preserving their order.
    pub fn entry_options_by_kind(&self) -> Vec<(&str, Vec<&LandscapeEntryOption>)> {
        let mut groups: Vec<(&str, Vec<&LandscapeEntryOption>)> = Vec::new();
        for option in &self.entry_options {
            match groups.last_mut() {
                Some((kind, options)) if *kind == option.kind => options.push(option),
                _ => groups.push((option.kind.as_str(), vec![option])),
            }
        }
        groups
    }
}

/// Affiliation linking the current user to a landscape entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAffiliation {
    /// Affiliation identifier.
    pub user_affiliation_id: Uuid,
    /// Landscape entry identifier.
    pub landscape_entry_id: Uuid,
    /// Landscape entry display name.
    pub entry_name: String,
    /// Landscape entry kind.
    pub entry_kind: String,
    /// Landscape entry logo URL.
    pub entry_logo_url: Option<String>,
    /// Landscape entry website URL.
    pub entry_website_url: Option<String>,
    /// Landscape entry GitHub URL.
    pub entry_github_url: Option<String>,
    /// Role the user holds in the entry.
    pub role: String,
}

impl UserAffiliation {
    /// Returns the display label for the affiliation role.
    pub fn role_label(&self) -> &str {
        role_label(&self.role)
    }

    /// Returns the entry's website URL, falling back to its GitHub URL.
    pub fn entry_url(&self) -> Option<&str> {
        self.entry_website_url.as_deref().or(self.entry_github_url.as_deref())
    }
}

/// Landscape entry option for the affiliation form select.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandscapeEntryOption {
    /// Landscape entry identifier.
    pub landscape_entry_id: Uuid,
    /// Landscape entry display name.
    pub name: String,
    /// Landscape entry kind.
    pub kind: String,
}

/// Form to add an affiliation or update the role of an existing one.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserAffiliationForm {
    /// Landscape entry being linked.
    #[garde(skip)]
    pub landscape_entry_id: Uuid,
    /// Role the user holds in the entry.
    #[garde(pattern(
        r"^(founder|co_founder|executive|maintainer|contributor|representative|other)$"
    ))]
    pub role: String,
}
