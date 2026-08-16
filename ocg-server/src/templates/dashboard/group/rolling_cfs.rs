//! Templates and payloads for a group's standing Call for Speakers.

use askama::Template;
use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    templates::dashboard::group::events::CfsSubmissionStatus,
    types::event::EventCfsLabel,
    validation::{MAX_LEN_DESCRIPTION, MAX_LEN_EVENT_LABELS_PER_SUBMISSION, trimmed_non_empty},
};

#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/group/rolling_cfs.html")]
pub(crate) struct Page {
    pub can_manage_events: bool,
    pub config: Config,
    pub events: Vec<AssignmentEvent>,
    pub statuses: Vec<CfsSubmissionStatus>,
    pub submissions: Vec<Submission>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct Config {
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub labels: Vec<EventCfsLabel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AssignmentEvent {
    pub event_id: Uuid,
    pub name: String,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub starts_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Assignment {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub assigned_at: DateTime<Utc>,
    pub event_id: Uuid,
    pub event_name: String,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub event_starts_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Submission {
    pub assignments: Vec<Assignment>,
    pub average_rating: Option<f64>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    pub group_cfs_submission_id: Uuid,
    pub labels: Vec<EventCfsLabel>,
    pub ratings_count: usize,
    pub speaker_name: String,
    pub status_id: String,
    pub status_name: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct ConfigInput {
    #[garde(length(max = MAX_LEN_DESCRIPTION))]
    pub description: Option<String>,
    #[serde(default)]
    #[garde(skip)]
    pub enabled: bool,
    #[serde(default)]
    #[garde(skip)]
    pub labels: Vec<EventCfsLabel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct SubmissionUpdate {
    #[serde(default)]
    #[garde(length(max = MAX_LEN_EVENT_LABELS_PER_SUBMISSION))]
    pub label_ids: Vec<Uuid>,
    #[garde(custom(trimmed_non_empty))]
    pub status_id: String,
    #[garde(length(max = MAX_LEN_DESCRIPTION))]
    pub action_required_message: Option<String>,
    #[garde(length(max = MAX_LEN_DESCRIPTION))]
    pub rating_comment: Option<String>,
    #[garde(range(min = 0, max = 5))]
    pub rating_stars: Option<i16>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct AssignmentInput {
    #[garde(skip)]
    pub event_id: Uuid,
}
