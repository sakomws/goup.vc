//! Templates and form types for group collaboration projects.

#![allow(clippy::ref_option, clippy::trivially_copy_pass_by_ref)]

use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::validation::{
    MAX_LEN_DATE, MAX_LEN_DESCRIPTION, MAX_LEN_DESCRIPTION_SHORT, MAX_LEN_ENTITY_NAME, MAX_LEN_L,
    MAX_LEN_M, optional_trimmed_string, trimmed_non_empty, trimmed_non_empty_opt,
};

/// Collaboration dashboard page.
#[derive(Debug, Clone, askama::Template, Serialize, Deserialize)]
#[template(path = "dashboard/group/projects.html")]
pub(crate) struct Page {
    pub can_manage_projects: bool,
    pub dashboard: CollaborationDashboard,
}

/// Full group collaboration dashboard.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct CollaborationDashboard {
    #[serde(default)]
    pub projects: Vec<CollaborationProject>,
    #[serde(default)]
    pub members: Vec<serde_json::Value>,
    #[serde(default)]
    pub goals: Vec<serde_json::Value>,
    #[serde(default)]
    pub tasks: Vec<serde_json::Value>,
    #[serde(default)]
    pub updates: Vec<serde_json::Value>,
    #[serde(default)]
    pub activities: Vec<serde_json::Value>,
    #[serde(default)]
    pub outcomes: Vec<serde_json::Value>,
    #[serde(default)]
    pub sessions: Vec<CollaborationSession>,
}

/// Project record shared by dashboard and public profiles.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CollaborationProject {
    pub collaboration_project_id: Uuid,
    pub group_id: Uuid,
    pub slug: String,
    pub name: String,
    pub summary: String,
    pub description: Option<String>,
    pub lifecycle: String,
    pub visibility: String,
    pub landscape_entry_id: Option<Uuid>,
    pub group_accelerator_cohort_id: Option<Uuid>,
    pub website_url: Option<String>,
    pub repository_url: Option<String>,
    pub cover_image_url: Option<String>,
    pub starts_on: Option<String>,
    pub target_ends_on: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

/// Public project profile payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PublicCollaborationProject {
    #[serde(flatten)]
    pub project: CollaborationProject,
    #[serde(default)]
    pub goals: Vec<serde_json::Value>,
    #[serde(default)]
    pub updates: Vec<serde_json::Value>,
    #[serde(default)]
    pub outcome_summary: serde_json::Value,
}

/// Expert office-hour session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CollaborationSession {
    pub collaboration_office_hour_session_id: Uuid,
    pub collaboration_project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub meeting_url: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub capacity: i32,
    pub status: String,
}

/// Organizer project creation form.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct CollaborationProjectInput {
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub slug: String,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_ENTITY_NAME))]
    pub name: String,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_DESCRIPTION_SHORT))]
    pub summary: String,
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION))]
    pub description: Option<String>,
    #[garde(custom(valid_lifecycle), length(max = MAX_LEN_M))]
    pub lifecycle: String,
    #[garde(custom(valid_visibility), length(max = MAX_LEN_M))]
    pub visibility: String,
    #[garde(skip)]
    pub landscape_entry_id: Option<Uuid>,
    #[garde(skip)]
    pub group_accelerator_cohort_id: Option<Uuid>,
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(url, length(max = MAX_LEN_L), custom(trimmed_non_empty_opt))]
    pub website_url: Option<String>,
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(url, length(max = MAX_LEN_L), custom(trimmed_non_empty_opt))]
    pub repository_url: Option<String>,
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(url, length(max = MAX_LEN_L), custom(trimmed_non_empty_opt))]
    pub cover_image_url: Option<String>,
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(custom(valid_date), length(max = MAX_LEN_DATE))]
    pub starts_on: Option<String>,
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(custom(valid_date), length(max = MAX_LEN_DATE))]
    pub target_ends_on: Option<String>,
}

/// Self-service member update form.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct CollaborationUpdateInput {
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_DESCRIPTION))]
    pub body: String,
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION_SHORT))]
    pub blockers: Option<String>,
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION_SHORT))]
    pub next_steps: Option<String>,
}

/// Self-service office-hour booking form.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct CollaborationBookingInput {
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION_SHORT))]
    pub question: Option<String>,
}

/// Organizer project invitation form.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct CollaborationInvitationInput {
    #[garde(skip)]
    pub user_id: Uuid,
    #[garde(custom(valid_invitation_role), length(max = MAX_LEN_M))]
    pub role: String,
}

fn valid_lifecycle(value: &impl AsRef<str>, _ctx: &()) -> garde::Result {
    match value.as_ref() {
        "proposed" | "active" | "paused" | "completed" | "archived" => Ok(()),
        _ => Err(garde::Error::new("invalid project lifecycle")),
    }
}

fn valid_visibility(value: &impl AsRef<str>, _ctx: &()) -> garde::Result {
    match value.as_ref() {
        "private" | "members" | "public" => Ok(()),
        _ => Err(garde::Error::new("invalid project visibility")),
    }
}

fn valid_invitation_role(value: &impl AsRef<str>, _ctx: &()) -> garde::Result {
    match value.as_ref() {
        "contributor" | "viewer" => Ok(()),
        _ => Err(garde::Error::new("invalid project invitation role")),
    }
}

fn valid_date(value: &Option<String>, _ctx: &()) -> garde::Result {
    value.as_deref().map_or(Ok(()), |value| {
        chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(|_| ())
            .map_err(|_| garde::Error::new("invalid date"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_project_enums() {
        assert!(valid_lifecycle(&"active", &()).is_ok());
        assert!(valid_lifecycle(&"unknown", &()).is_err());
        assert!(valid_visibility(&"public", &()).is_ok());
        assert!(valid_visibility(&"secret", &()).is_err());
    }
}
