//! GTM pipeline types for alliance and group leads.

#![allow(clippy::ref_option, clippy::trivially_copy_pass_by_ref)]

use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::{
    impl_pagination_and_raw_query,
    templates::dashboard,
    types::pagination::{Pagination, ToRawQuery},
    validation::{
        MAX_LEN_DESCRIPTION, MAX_LEN_ENTITY_NAME, MAX_LEN_L, MAX_LEN_M, MAX_PAGINATION_LIMIT,
        optional_trimmed_string, trimmed_non_empty, trimmed_non_empty_opt,
    },
};

/// Allowed GTM lead kinds.
pub(crate) const GTM_KINDS: [&str; 5] = ["sponsor", "startup", "investor", "speaker", "organizer"];

/// Pipeline stages, including the won/lost fork.
pub(crate) const GTM_STAGES: [&str; 10] = [
    "lead_generation",
    "reachout",
    "get_response",
    "qualification",
    "proposal",
    "negotiation",
    "won",
    "lost",
    "delivered",
    "renewal",
];

/// Draft-only agent identifiers.
pub(crate) const GTM_AGENTS: [&str; 9] = [
    "lead_generation",
    "reachout",
    "get_response",
    "qualification",
    "proposal",
    "negotiation",
    "won_lost",
    "delivered",
    "renewal",
];

const GTM_SOURCES: [&str; 4] = ["manual", "landscape", "member", "discovery"];

/// Dashboard list filters.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub(crate) struct GtmLeadFilters {
    /// Free-text search.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(length(max = MAX_LEN_M))]
    pub query: Option<String>,
    /// Filter by lead kind.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(length(max = MAX_LEN_M), custom(valid_gtm_kind_opt))]
    pub kind: Option<String>,
    /// Filter by pipeline stage.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(length(max = MAX_LEN_M), custom(valid_gtm_stage_opt))]
    pub stage: Option<String>,
    /// Filter by assigned group.
    #[garde(skip)]
    pub group_id: Option<Uuid>,
    /// Filter by owner.
    #[garde(skip)]
    pub owner_user_id: Option<Uuid>,
    /// Page size.
    #[serde(default = "dashboard::default_limit")]
    #[garde(range(max = MAX_PAGINATION_LIMIT))]
    pub limit: Option<usize>,
    /// Page offset.
    #[serde(default = "dashboard::default_offset")]
    #[garde(skip)]
    pub offset: Option<usize>,
}

impl_pagination_and_raw_query!(GtmLeadFilters, limit, offset);

/// Paginated lead list plus stage counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct GtmLeadList {
    /// Matching leads.
    #[serde(default)]
    pub leads: Vec<GtmLead>,
    /// Total matching leads.
    #[serde(default)]
    pub total: usize,
    /// Counts keyed by stage.
    #[serde(default)]
    pub stage_counts: serde_json::Map<String, serde_json::Value>,
}

/// Stored GTM lead.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct GtmLead {
    /// Lead identifier.
    pub gtm_lead_id: Uuid,
    /// Owning alliance.
    pub alliance_id: Uuid,
    /// Optional group assignment.
    pub group_id: Option<Uuid>,
    /// Lead kind.
    pub kind: String,
    /// Current stage.
    pub stage: String,
    /// Contact or company display name.
    pub name: String,
    /// Organization name.
    pub org_name: Option<String>,
    /// Contact email.
    pub email: Option<String>,
    /// Website.
    pub website_url: Option<String>,
    /// `LinkedIn` profile URL.
    pub linkedin_url: Option<String>,
    /// Linked landscape entry.
    pub landscape_entry_id: Option<Uuid>,
    /// Linked platform user.
    pub user_id: Option<Uuid>,
    /// Linked group sponsor after a win.
    pub group_sponsor_id: Option<Uuid>,
    /// Organizer who owns the deal.
    pub owner_user_id: Option<Uuid>,
    /// Qualification score.
    pub score: Option<i32>,
    /// Estimated deal value in cents.
    pub estimated_value_cents: Option<i64>,
    /// Currency code.
    pub currency: Option<String>,
    /// Next follow-up time.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub next_action_at: Option<DateTime<Utc>>,
    /// Renewal date.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub renewal_at: Option<DateTime<Utc>>,
    /// Lost reason.
    pub lost_reason: Option<String>,
    /// How the lead was created.
    pub source: String,
    /// Free-form notes.
    pub notes: Option<String>,
    /// Structured extras.
    #[serde(default)]
    pub payload: serde_json::Value,
    /// Activity timeline (detail view).
    #[serde(default)]
    pub activities: Vec<GtmLeadActivity>,
    /// Agent drafts (detail view).
    #[serde(default)]
    pub drafts: Vec<GtmAgentDraft>,
    /// Created at.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Updated at.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Timeline row on a lead.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct GtmLeadActivity {
    /// Activity identifier.
    pub gtm_lead_activity_id: Uuid,
    /// Lead identifier.
    pub gtm_lead_id: Option<Uuid>,
    /// Human actor, if any.
    pub actor_user_id: Option<Uuid>,
    /// Agent identifier, if any.
    pub agent_id: Option<String>,
    /// Activity kind.
    pub kind: String,
    /// Display body.
    pub body: Option<String>,
    /// Structured details.
    #[serde(default)]
    pub details: serde_json::Value,
    /// Created at.
    #[serde(default, with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

/// Draft produced by an agent.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct GtmAgentDraft {
    /// Draft identifier.
    pub gtm_agent_draft_id: Uuid,
    /// Owning alliance.
    pub alliance_id: Uuid,
    /// Optional group.
    pub group_id: Option<Uuid>,
    /// Target lead, if any.
    pub gtm_lead_id: Option<Uuid>,
    /// Agent that produced the draft.
    pub agent_id: String,
    /// Review status.
    pub status: String,
    /// Short title.
    pub title: String,
    /// Email or proposal body.
    pub body: String,
    /// Suggested next stage.
    pub suggested_stage: Option<String>,
    /// Structured extras.
    #[serde(default)]
    pub payload: serde_json::Value,
    /// Created at.
    #[serde(default, with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    /// Reviewed at.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub reviewed_at: Option<DateTime<Utc>>,
    /// Reviewer.
    pub reviewed_by: Option<Uuid>,
}

/// Manual create/update input.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub(crate) struct GtmLeadInput {
    /// Optional group assignment.
    #[garde(skip)]
    pub group_id: Option<Uuid>,
    /// Lead kind.
    #[garde(custom(valid_gtm_kind))]
    pub kind: String,
    /// Display name.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_ENTITY_NAME))]
    pub name: String,
    /// Organization.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_ENTITY_NAME))]
    pub org_name: Option<String>,
    /// Email.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(email, length(max = MAX_LEN_M), custom(trimmed_non_empty_opt))]
    pub email: Option<String>,
    /// Website.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(url, length(max = MAX_LEN_M), custom(trimmed_non_empty_opt))]
    pub website_url: Option<String>,
    /// `LinkedIn` profile URL.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(url, length(max = MAX_LEN_M), custom(trimmed_non_empty_opt))]
    pub linkedin_url: Option<String>,
    /// Owner.
    #[garde(skip)]
    pub owner_user_id: Option<Uuid>,
    /// Notes.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION))]
    pub notes: Option<String>,
    /// Source.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(length(max = MAX_LEN_M), custom(valid_gtm_source_opt))]
    pub source: Option<String>,
}

/// Human stage move.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub(crate) struct GtmLeadTransitionInput {
    /// Target stage.
    #[garde(custom(valid_gtm_stage))]
    pub stage: String,
    /// Optional lost reason.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_L))]
    pub lost_reason: Option<String>,
}

/// Request to run one agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub(crate) struct GtmRunAgentInput {
    /// Agent to run.
    #[garde(custom(valid_gtm_agent))]
    pub agent_id: String,
    /// Optional pasted reply used by Get Response.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_L))]
    pub reply: Option<String>,
}

/// Human review of an agent draft.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub(crate) struct GtmReviewDraftInput {
    /// approved or rejected.
    #[garde(custom(valid_gtm_review_status))]
    pub status: String,
    /// Optional edited body.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_L))]
    pub body: Option<String>,
    /// Optional edited suggested stage.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(length(max = MAX_LEN_M), custom(valid_gtm_stage_opt))]
    pub suggested_stage: Option<String>,
    /// Reviewer note.
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_L))]
    pub note: Option<String>,
    /// Create a sponsor record on won.
    #[serde(default)]
    #[garde(skip)]
    pub create_sponsor: bool,
    /// Create a landscape entry on won.
    #[serde(default)]
    #[garde(skip)]
    pub create_landscape_entry: bool,
    /// Invite as organizer on won.
    #[serde(default)]
    #[garde(skip)]
    pub invite_organizer: bool,
}

/// Result of reviewing a draft.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct GtmReviewDraftResult {
    /// Draft identifier.
    pub gtm_agent_draft_id: Uuid,
    /// New status.
    pub status: String,
    /// Leads created from a lead-gen draft.
    #[serde(default)]
    pub created_lead_ids: Vec<Uuid>,
    /// Optional won side effects.
    #[serde(default)]
    pub side_effects: serde_json::Value,
    /// Related lead.
    pub gtm_lead_id: Option<Uuid>,
    /// Agent identifier.
    pub agent_id: Option<String>,
    /// Approved body.
    pub body: Option<String>,
    /// Applied stage.
    pub suggested_stage: Option<String>,
}

/// Candidate rows returned by lead generation.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct GtmLeadCandidates {
    /// Suggested leads.
    #[serde(default)]
    pub candidates: Vec<serde_json::Value>,
}

/// Draft list wrapper.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct GtmAgentDraftList {
    /// Drafts.
    #[serde(default)]
    pub drafts: Vec<GtmAgentDraft>,
}

fn valid_gtm_kind(value: &str, _: &()) -> garde::Result {
    if GTM_KINDS.contains(&value) {
        Ok(())
    } else {
        Err(garde::Error::new("invalid gtm kind"))
    }
}

fn valid_gtm_kind_opt(value: &Option<String>, _: &()) -> garde::Result {
    match value {
        Some(kind) => valid_gtm_kind(kind, &()),
        None => Ok(()),
    }
}

fn valid_gtm_stage(value: &str, _: &()) -> garde::Result {
    if GTM_STAGES.contains(&value) {
        Ok(())
    } else {
        Err(garde::Error::new("invalid gtm stage"))
    }
}

fn valid_gtm_stage_opt(value: &Option<String>, _: &()) -> garde::Result {
    match value {
        Some(stage) => valid_gtm_stage(stage, &()),
        None => Ok(()),
    }
}

fn valid_gtm_agent(value: &str, _: &()) -> garde::Result {
    if GTM_AGENTS.contains(&value) {
        Ok(())
    } else {
        Err(garde::Error::new("invalid gtm agent"))
    }
}

fn valid_gtm_source_opt(value: &Option<String>, _: &()) -> garde::Result {
    match value {
        Some(source) if GTM_SOURCES.contains(&source.as_str()) => Ok(()),
        Some(_) => Err(garde::Error::new("invalid gtm source")),
        None => Ok(()),
    }
}

fn valid_gtm_review_status(value: &str, _: &()) -> garde::Result {
    if matches!(value, "approved" | "rejected") {
        Ok(())
    } else {
        Err(garde::Error::new("invalid gtm review status"))
    }
}

/// Recommended next stage for an agent given the current stage.
pub(crate) fn suggested_stage_for_agent(
    agent_id: &str,
    current_stage: &str,
) -> Option<&'static str> {
    let preferred = match agent_id {
        "lead_generation" => "lead_generation",
        "reachout" => "reachout",
        "get_response" => "get_response",
        "qualification" => "qualification",
        "proposal" => "proposal",
        "negotiation" => "negotiation",
        "won_lost" => "won",
        "delivered" => "delivered",
        "renewal" => "renewal",
        _ => return None,
    };
    if current_stage == preferred || agent_may_suggest(current_stage, preferred) {
        Some(preferred)
    } else {
        next_legal_agent_stage(current_stage).or(Some(preferred))
    }
}

fn agent_may_suggest(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("lead_generation" | "renewal" | "lost", "reachout")
            | ("reachout", "get_response")
            | ("get_response", "qualification")
            | ("qualification", "proposal" | "lost")
            | ("proposal", "negotiation")
            | ("negotiation", "won" | "lost")
            | ("won", "delivered")
            | ("delivered", "renewal")
    )
}

fn next_legal_agent_stage(from: &str) -> Option<&'static str> {
    match from {
        "reachout" => Some("get_response"),
        "get_response" => Some("qualification"),
        "qualification" => Some("proposal"),
        "proposal" => Some("negotiation"),
        "negotiation" => Some("won"),
        "won" => Some("delivered"),
        "delivered" => Some("renewal"),
        "lead_generation" | "renewal" | "lost" => Some("reachout"),
        _ => None,
    }
}

/// Display label for a stage.
pub(crate) fn stage_label(stage: &str) -> &'static str {
    match stage {
        "lead_generation" => "Lead generation",
        "reachout" => "Reachout",
        "get_response" => "Get response",
        "qualification" => "Qualification",
        "proposal" => "Proposal",
        "negotiation" => "Negotiation",
        "won" => "Won",
        "lost" => "Lost",
        "delivered" => "Delivered",
        "renewal" => "Renewal",
        _ => "Unknown",
    }
}
