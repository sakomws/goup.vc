//! Event survey payloads and aggregate dashboard data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{event::EventSummary, questionnaire::QuestionnaireQuestion};

/// Authenticated event survey rendered for one eligible audience member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSurvey {
    pub event: EventSummary,
    pub audience: String,
    pub questions: Vec<QuestionnaireQuestion>,
    pub submitted: bool,
}

/// Aggregate metrics for one event survey audience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSurveyMetrics {
    pub audience: String,
    pub eligible: i64,
    pub responses: i64,
    pub response_rate: f64,
    pub promoters: i64,
    pub passives: i64,
    pub detractors: i64,
    pub nps_score: Option<f64>,
    pub average_rating: Option<f64>,
}

/// Anonymous organizer review row. Respondent identity is intentionally absent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSurveyResponseReview {
    pub audience: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub submitted_at: DateTime<Utc>,
    pub nps: Option<i32>,
    pub rating: Option<i32>,
    pub comment: Option<String>,
}

/// Organizer survey dashboard payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSurveyDashboard {
    pub metrics: Vec<EventSurveyMetrics>,
    pub responses: Vec<EventSurveyResponseReview>,
}
