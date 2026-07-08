//! Mock interview practice types.

use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::validation::{
    MAX_LEN_DESCRIPTION_SHORT, MAX_LEN_M, trimmed_non_empty, trimmed_non_empty_opt,
};

/// User role intent for mock interviews.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RoleIntent {
    Interviewee,
    Interviewer,
    Both,
}

/// Timezone region bucket.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TimezoneRegion {
    Aze,
    Eu,
    #[serde(rename = "usa_canada")]
    UsaCanada,
    Asia,
    Other,
}

/// Seniority level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Seniority {
    Junior,
    Mid,
    Senior,
    #[serde(rename = "staff_plus")]
    StaffPlus,
}

/// Supported interview types.
pub(crate) const INTERVIEW_TYPES: &[&str] = &[
    "swe",
    "ai_ml",
    "pm",
    "devops_cloud",
    "startup_cofounder",
    "behavioral",
    "other",
];

/// Supported target company types.
pub(crate) const TARGET_COMPANY_TYPES: &[&str] = &[
    "ai_labs_faang",
    "ai_startup",
    "enterprise",
    "remote_global",
];

/// Onboarding/profile form input.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct MockInterviewProfileInput {
    #[garde(skip)]
    pub role_intent: RoleIntent,
    #[garde(skip)]
    pub timezone_region: TimezoneRegion,
    #[garde(skip)]
    pub seniority: Seniority,
    #[garde(length(max = 500))]
    pub interview_types: Option<String>,
    #[garde(length(max = 500))]
    pub target_company_types: Option<String>,
    #[garde(length(max = MAX_LEN_DESCRIPTION_SHORT))]
    pub availability_slots: Option<String>,
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_M))]
    pub linkedin_url: Option<String>,
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_M))]
    pub github_url: Option<String>,
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_M))]
    pub resume_url: Option<String>,
    #[garde(skip)]
    pub enabled: Option<bool>,
}

/// Parse comma-separated values from onboarding forms.
pub(crate) fn parse_csv_values(input: Option<&str>) -> Vec<String> {
    input
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .take(12)
        .map(str::to_owned)
        .collect()
}

/// Build JSON payload for profile upsert SQL function.
pub(crate) fn profile_input_to_json(input: &MockInterviewProfileInput) -> serde_json::Value {
    let availability_slots = input
        .availability_slots
        .as_deref()
        .and_then(|raw| serde_json::from_str(raw).ok())
        .unwrap_or(serde_json::Value::Array(vec![]));

    serde_json::json!({
        "role_intent": input.role_intent,
        "timezone_region": input.timezone_region,
        "seniority": input.seniority,
        "interview_types": parse_csv_values(input.interview_types.as_deref()),
        "target_company_types": parse_csv_values(input.target_company_types.as_deref()),
        "availability_slots": availability_slots,
        "linkedin_url": input.linkedin_url,
        "github_url": input.github_url,
        "resume_url": input.resume_url,
        "enabled": input.enabled.unwrap_or(true),
    })
}

/// Match search filters.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub(crate) struct MockInterviewMatchFilters {
    #[garde(length(max = MAX_LEN_M))]
    pub interview_type: Option<String>,
    #[garde(range(max = 10))]
    pub limit: Option<usize>,
}

/// Request creation input.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct MockInterviewRequestInput {
    #[garde(skip)]
    pub interviewer_user_id: Uuid,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub interview_type: String,
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION_SHORT))]
    pub message: Option<String>,
}

/// Request response input.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct MockInterviewRespondInput {
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub action: String,
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_M))]
    pub meeting_url: Option<String>,
}

/// Interviewer feedback input.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct MockInterviewInterviewerFeedbackInput {
    #[garde(range(min = 1, max = 5))]
    pub communication: i16,
    #[garde(range(min = 1, max = 5))]
    pub technical_depth: i16,
    #[garde(range(min = 1, max = 5))]
    pub problem_solving: i16,
    #[garde(range(min = 1, max = 5))]
    pub role_readiness: i16,
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION_SHORT))]
    pub suggested_next_steps: Option<String>,
}

/// Interviewee feedback input.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct MockInterviewIntervieweeFeedbackInput {
    #[garde(range(min = 1, max = 5))]
    pub helpfulness: i16,
    #[garde(range(min = 1, max = 5))]
    pub feedback_quality: i16,
    #[garde(skip)]
    pub would_recommend: Option<bool>,
}

/// Member profile for mock interviews.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MockInterviewProfile {
    pub user_id: Uuid,
    pub role_intent: RoleIntent,
    pub timezone_region: TimezoneRegion,
    pub seniority: Seniority,
    #[serde(default)]
    pub interview_types: Vec<String>,
    #[serde(default)]
    pub target_company_types: Vec<String>,
    #[serde(default)]
    pub availability_slots: Value,
    pub linkedin_url: Option<String>,
    pub github_url: Option<String>,
    pub resume_url: Option<String>,
    pub enabled: bool,
    #[serde(default)]
    pub reputation_score: f64,
    #[serde(default)]
    pub completed_sessions: i32,
    #[serde(default)]
    pub interviewer_badge: bool,
    pub username: String,
    pub name: Option<String>,
    pub photo_url: Option<String>,
    pub title: Option<String>,
    pub company: Option<String>,
    #[serde(default)]
    pub match_score: Option<i32>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Match search output.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct MockInterviewMatchesOutput {
    #[serde(default)]
    pub matches: Vec<MockInterviewProfile>,
    #[serde(default)]
    pub total: usize,
}

/// Mock interview request.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MockInterviewRequest {
    pub mock_interview_request_id: Uuid,
    pub interviewee_user_id: Uuid,
    pub interviewer_user_id: Uuid,
    pub interview_type: String,
    pub message: Option<String>,
    pub status: String,
    pub interviewee_username: String,
    pub interviewee_name: Option<String>,
    pub interviewee_photo_url: Option<String>,
    pub interviewer_username: String,
    pub interviewer_name: Option<String>,
    pub interviewer_photo_url: Option<String>,
    #[serde(default)]
    pub mock_interview_session_id: Option<Uuid>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub responded_at: Option<DateTime<Utc>>,
}

/// Session feedback record.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MockInterviewFeedback {
    pub mock_interview_feedback_id: Uuid,
    pub reviewer_user_id: Uuid,
    pub reviewee_user_id: Uuid,
    pub reviewer_role: String,
    pub communication: Option<i16>,
    pub technical_depth: Option<i16>,
    pub problem_solving: Option<i16>,
    pub role_readiness: Option<i16>,
    pub helpfulness: Option<i16>,
    pub feedback_quality: Option<i16>,
    pub would_recommend: Option<bool>,
    pub suggested_next_steps: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

/// Mock interview session details.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MockInterviewSession {
    pub mock_interview_session_id: Uuid,
    pub mock_interview_request_id: Uuid,
    pub meeting_url: Option<String>,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub scheduled_at: Option<DateTime<Utc>>,
    pub status: String,
    pub request: MockInterviewRequest,
    #[serde(default)]
    pub feedback: Vec<MockInterviewFeedback>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub completed_at: Option<DateTime<Utc>>,
}

/// Human-readable labels for UI.
pub(crate) fn label_interview_type(value: &str) -> &str {
    match value {
        "swe" => "Software Engineering",
        "ai_ml" => "AI / ML",
        "pm" => "Product Management",
        "devops_cloud" => "DevOps / Cloud",
        "startup_cofounder" => "Startup / Co-founder",
        "behavioral" => "Behavioral",
        _ => "Other",
    }
}

pub(crate) fn label_company_type(value: &str) -> &str {
    match value {
        "ai_labs_faang" => "AI Labs / FAANG",
        "ai_startup" => "AI Startup",
        "enterprise" => "Enterprise",
        "remote_global" => "Remote / Global",
        _ => value,
    }
}

pub(crate) fn label_timezone(value: &TimezoneRegion) -> &'static str {
    match value {
        TimezoneRegion::Aze => "AZE",
        TimezoneRegion::Eu => "EU",
        TimezoneRegion::UsaCanada => "USA / Canada",
        TimezoneRegion::Asia => "Asia",
        TimezoneRegion::Other => "Other",
    }
}

pub(crate) fn label_seniority(value: &Seniority) -> &'static str {
    match value {
        Seniority::Junior => "Junior",
        Seniority::Mid => "Mid",
        Seniority::Senior => "Senior",
        Seniority::StaffPlus => "Staff+",
    }
}

pub(crate) fn label_role_intent(value: &RoleIntent) -> &'static str {
    match value {
        RoleIntent::Interviewee => "Interviewee",
        RoleIntent::Interviewer => "Interviewer",
        RoleIntent::Both => "Both",
    }
}
