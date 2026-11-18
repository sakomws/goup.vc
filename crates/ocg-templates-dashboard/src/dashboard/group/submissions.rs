//! Templates and types for event CFS submissions in the group dashboard.

use askama::Template;
use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::dashboard;
use ocg_core::types::{
    event::EventCfsLabel,
    pagination::{self, Pagination, ToRawQuery},
    user::UserSummary,
};
use ocg_core::validation::{
    MAX_LEN_DESCRIPTION, MAX_LEN_EVENT_LABELS_PER_EVENT, MAX_LEN_EVENT_LABELS_PER_SUBMISSION,
    MAX_LEN_M, MAX_LEN_SORT_KEY, MAX_PAGINATION_LIMIT, trimmed_non_empty, trimmed_non_empty_opt,
};
use ocg_templates_common::{filters, helpers::user_initials};

use super::events::CfsSubmissionStatus;

// Pages templates.

/// List submissions page template for an event.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/group/event_submissions_list.html")]
pub struct ListPage {
    /// Whether the current user can manage events.
    pub can_manage_events: bool,
    /// Whether the event has been canceled.
    pub event_canceled: bool,
    /// Whether the event has a venue name for the schedule email.
    pub event_has_venue: bool,
    /// Event CFS labels available for filtering and submission updates.
    pub event_cfs_labels: Vec<CfsLabelFilterOption>,
    /// Event identifier.
    pub event_id: Uuid,
    /// Pagination navigation links.
    pub navigation_links: pagination::NavigationLinks,
    /// Whether submissions are filtered to entries without any labels.
    pub no_labels: bool,
    /// URL used to refresh the submissions list.
    pub refresh_url: String,
    /// Selected CFS label identifiers used to filter submissions.
    pub selected_event_cfs_label_ids: Option<Vec<Uuid>>,
    /// Sort option used to order submissions.
    pub sort: String,
    /// Text search query used to filter submissions.
    pub ts_query: Option<String>,
    /// Submission status options.
    pub statuses: Vec<CfsSubmissionStatus>,
    /// List of submissions.
    pub submissions: Vec<CfsSubmission>,
    /// Total number of submissions.
    pub total: usize,
    /// Number of active submissions without any label assigned.
    pub unlabeled_submissions_count: usize,

    /// Number of results per page.
    pub limit: Option<usize>,
    /// Pagination offset for results.
    pub offset: Option<usize>,
}

// Types.

/// Event CFS label shown in the submissions filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfsLabelFilterOption {
    /// Label color.
    pub color: String,
    /// Event CFS label identifier.
    pub event_cfs_label_id: Option<Uuid>,
    /// Label name.
    pub name: String,
    /// Number of active submissions assigned to the label.
    pub submissions_count: usize,
}

/// Session proposal summary for a submission.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfsSessionProposal {
    /// Session proposal identifier.
    pub session_proposal_id: Uuid,
    /// Proposal title.
    pub title: String,

    /// Co-speaker information.
    pub co_speaker: Option<UserSummary>,
    /// Proposal description.
    pub description: Option<String>,
    /// Duration in minutes.
    pub duration_minutes: Option<i32>,
    /// Session proposal level identifier.
    pub session_proposal_level_id: Option<String>,
    /// Session proposal level display name.
    pub session_proposal_level_name: Option<String>,
}

/// Event submission summary information.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfsSubmission {
    /// Submission identifier.
    pub cfs_submission_id: Uuid,
    /// Submission creation time.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    /// Labels assigned to the submission.
    pub labels: Vec<EventCfsLabel>,
    /// Speaker prep emails already sent for this submission.
    #[serde(default)]
    pub prep_emails: Vec<String>,
    /// Ratings for this submission.
    pub ratings: Vec<CfsSubmissionRating>,
    /// Total number of ratings.
    pub ratings_count: usize,
    /// Session proposal summary information.
    pub session_proposal: CfsSessionProposal,
    /// Speaker information.
    pub speaker: UserSummary,
    /// Submission status identifier.
    pub status_id: String,
    /// Submission status name.
    pub status_name: String,

    /// Action required message for the speaker.
    pub action_required_message: Option<String>,
    /// Average stars across all ratings.
    pub average_rating: Option<f64>,
    /// Linked session identifier.
    pub linked_session_id: Option<Uuid>,
    /// Start time of the linked session.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub session_starts_at: Option<DateTime<Utc>>,
    /// Reviewer information.
    pub reviewed_by: Option<UserSummary>,
    /// Submission last update time.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl CfsSubmission {
    /// Whether the named speaker prep email has already been sent.
    pub fn prep_email_sent(&self, step: &str) -> bool {
        self.prep_emails.iter().any(|sent| sent == step)
    }

    /// Whether this talk is on the event agenda.
    pub fn has_session(&self) -> bool {
        self.linked_session_id.is_some()
    }

    /// Whether the talk has a track and a session time.
    pub fn has_schedule_details(&self) -> bool {
        self.session_starts_at.is_some() && !self.labels.is_empty()
    }
}

/// Facts used to compose one speaker prep email.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfsSubmissionPrepEmailContext {
    /// Speaker who submitted the proposal.
    pub speaker_user_id: Uuid,
    /// Submission status identifier.
    pub status_id: String,
    /// Proposal title.
    pub title: String,

    /// Co-speaker on the proposal.
    pub co_speaker_user_id: Option<Uuid>,
    /// Linked session identifier.
    pub linked_session_id: Option<Uuid>,
    /// Speaker prep emails already sent.
    #[serde(default)]
    pub prep_emails: Vec<String>,
    /// Start time of the linked session.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub session_starts_at: Option<DateTime<Utc>>,
    /// Track names assigned to the submission.
    #[serde(default)]
    pub tracks: Vec<String>,
    /// Event venue name.
    pub venue_name: Option<String>,
}

impl CfsSubmissionPrepEmailContext {
    /// Whether the named speaker prep email has already been sent.
    pub fn prep_email_sent(&self, step: &str) -> bool {
        self.prep_emails.iter().any(|sent| sent == step)
    }

    /// Whether this talk is on the event agenda.
    pub fn has_session(&self) -> bool {
        self.linked_session_id.is_some()
    }
}

/// Notification data for a submission update.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfsSubmissionNotificationData {
    /// Submission status identifier.
    pub status_id: String,
    /// Submission status name.
    pub status_name: String,
    /// User identifier.
    pub user_id: Uuid,

    /// Action required message for the speaker.
    pub action_required_message: Option<String>,
}

/// Submission rating details.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfsSubmissionRating {
    /// Reviewer information.
    pub reviewer: UserSummary,
    /// Number of stars assigned by the reviewer.
    pub stars: i16,

    /// Optional rating comments.
    pub comments: Option<String>,
}

/// Filter parameters for submissions pagination.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct CfsSubmissionsFilters {
    /// Labels to filter by.
    #[garde(length(max = MAX_LEN_EVENT_LABELS_PER_EVENT))]
    pub label_ids: Option<Vec<Uuid>>,
    /// Whether to filter submissions with no labels assigned.
    #[garde(skip)]
    pub no_labels: Option<bool>,
    /// Number of results per page.
    #[serde(default = "dashboard::default_limit")]
    #[garde(range(max = MAX_PAGINATION_LIMIT))]
    pub limit: Option<usize>,
    /// Pagination offset for results.
    #[serde(default = "dashboard::default_offset")]
    #[garde(skip)]
    pub offset: Option<usize>,
    /// Sort option used to order submissions.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_SORT_KEY))]
    pub sort: Option<String>,
    /// Search query for speaker name, username, company, title, or proposal title.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_M))]
    pub ts_query: Option<String>,
}

ocg_core::impl_pagination_and_raw_query!(CfsSubmissionsFilters, limit, offset);

/// Paginated submissions response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfsSubmissionsOutput {
    /// Submission counts grouped by event CFS label.
    pub label_counts: Vec<CfsLabelSubmissionCount>,
    /// List of submissions.
    pub submissions: Vec<CfsSubmission>,
    /// Total number of submissions.
    pub total: usize,
    /// Number of active submissions without any label assigned.
    #[serde(default)]
    pub unlabeled_count: usize,
}

/// Number of active submissions assigned to an event CFS label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfsLabelSubmissionCount {
    /// Event CFS label identifier.
    pub event_cfs_label_id: Uuid,
    /// Number of active submissions assigned to the label.
    pub submissions_count: usize,
}

/// Submission update payload.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CfsSubmissionUpdate {
    /// Labels assigned to the submission.
    #[serde(default)]
    #[garde(length(max = MAX_LEN_EVENT_LABELS_PER_SUBMISSION))]
    pub label_ids: Vec<Uuid>,
    /// Submission status identifier.
    #[garde(custom(trimmed_non_empty))]
    pub status_id: String,

    /// Action required message for the speaker.
    #[garde(length(max = MAX_LEN_DESCRIPTION))]
    pub action_required_message: Option<String>,
    /// Optional comment attached to the reviewer's rating.
    #[garde(length(max = MAX_LEN_DESCRIPTION))]
    pub rating_comment: Option<String>,
    /// Reviewer stars value, where `0` clears the existing rating.
    #[garde(range(min = 0, max = 5))]
    pub rating_stars: Option<i16>,
}
