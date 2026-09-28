//! Unified opportunity board types.

use chrono::{DateTime, Utc};
use garde::Validate;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::{
    templates::dashboard,
    types::pagination::{Pagination, ToRawQuery},
    validation::{
        MAX_LEN_DESCRIPTION, MAX_LEN_DESCRIPTION_SHORT, MAX_LEN_ENTITY_NAME, MAX_LEN_M,
        MAX_LEN_TAG, MAX_PAGINATION_LIMIT, trimmed_non_empty, trimmed_non_empty_opt,
    },
};

/// Filters shared by the board, global search, saved searches, CSV and MCP.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct OpportunityFilters {
    #[garde(length(max = MAX_LEN_M))]
    pub query: Option<String>,
    #[garde(length(max = MAX_LEN_ENTITY_NAME))]
    pub kind: Option<String>,
    #[garde(length(max = MAX_LEN_M))]
    pub location: Option<String>,
    #[garde(skip)]
    pub remote: Option<bool>,
    #[serde(default, skip_deserializing)]
    #[garde(skip)]
    pub include_members_only: bool,
    #[serde(default = "default_limit")]
    #[garde(range(max = MAX_PAGINATION_LIMIT))]
    pub limit: Option<usize>,
    #[serde(default = "dashboard::default_offset")]
    #[garde(skip)]
    pub offset: Option<usize>,
}

impl Default for OpportunityFilters {
    fn default() -> Self {
        Self {
            query: None,
            kind: None,
            location: None,
            remote: None,
            include_members_only: false,
            limit: default_limit(),
            offset: dashboard::default_offset(),
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
fn default_limit() -> Option<usize> {
    Some(20)
}

crate::impl_pagination_and_raw_query!(OpportunityFilters, limit, offset);

/// Dashboard pagination.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct DashboardOpportunityFilters {
    #[serde(default = "dashboard::default_limit")]
    #[garde(range(max = MAX_PAGINATION_LIMIT))]
    pub limit: Option<usize>,
    #[serde(default = "dashboard::default_offset")]
    #[garde(skip)]
    pub offset: Option<usize>,
}

impl Default for DashboardOpportunityFilters {
    fn default() -> Self {
        Self {
            limit: dashboard::default_limit(),
            offset: dashboard::default_offset(),
        }
    }
}

crate::impl_pagination_and_raw_query!(DashboardOpportunityFilters, limit, offset);

/// Native grant, funding, partnership or research form.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct OpportunityInput {
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_ENTITY_NAME))]
    pub kind: String,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_ENTITY_NAME))]
    pub title: String,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_ENTITY_NAME))]
    pub organization_name: String,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_DESCRIPTION_SHORT))]
    pub summary: String,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_DESCRIPTION))]
    pub description: String,
    #[garde(custom(application_url), length(max = MAX_LEN_M))]
    pub apply_url: String,
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_M))]
    pub location: Option<String>,
    #[garde(skip)]
    pub remote: Option<bool>,
    #[garde(skip)]
    pub members_only: Option<bool>,
    #[garde(length(max = MAX_LEN_M))]
    pub tags: Option<String>,
    #[garde(skip)]
    pub opens_at: Option<String>,
    #[garde(skip)]
    pub closes_at: Option<String>,
}

impl OpportunityInput {
    pub(crate) fn valid_kind(&self) -> bool {
        matches!(
            self.kind.as_str(),
            "grant" | "funding" | "partnership" | "research"
        )
    }
}

/// Saved-search form. Activation is deliberately a separate action so users can
/// preview results before enabling notifications.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct OpportunitySavedSearchInput {
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_ENTITY_NAME))]
    pub name: String,
    #[garde(custom(trimmed_non_empty), length(max = 16))]
    pub frequency: String,
    #[garde(length(max = MAX_LEN_M))]
    pub query: Option<String>,
    #[garde(length(max = MAX_LEN_ENTITY_NAME))]
    pub kind: Option<String>,
    #[garde(length(max = MAX_LEN_M))]
    pub location: Option<String>,
    #[garde(skip)]
    pub remote: Option<bool>,
}

impl OpportunitySavedSearchInput {
    pub(crate) fn valid_frequency(&self) -> bool {
        matches!(self.frequency.as_str(), "daily" | "weekly")
    }

    pub(crate) fn filters(&self) -> Value {
        json!({
            "query": self.query,
            "kind": self.kind,
            "location": self.location,
            "remote": self.remote,
            "limit": 20,
            "offset": 0
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct OpportunitiesOutput {
    pub opportunities: Vec<OpportunitySummary>,
    pub total: usize,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OpportunitySummary {
    pub source_id: Uuid,
    pub source_kind: String,
    pub kind: String,
    pub title: String,
    pub organization_name: String,
    pub summary: String,
    pub description: String,
    pub apply_url: String,
    pub location: Option<String>,
    pub remote: bool,
    pub members_only: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    pub published: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub opens_at: Option<DateTime<Utc>>,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub closes_at: Option<DateTime<Utc>>,
    pub posted_by_user_id: Option<Uuid>,
    pub poster_username: Option<String>,
    pub poster_name: Option<String>,
}

impl OpportunitySummary {
    pub(crate) fn details_url(&self) -> String {
        format!("/opportunities/{}/{}", self.source_kind, self.source_id)
    }

    pub(crate) fn kind_label(&self) -> &str {
        match self.kind.as_str() {
            "cfs" => "Call for speakers",
            "funding" => "Funding",
            "grant" => "Grant",
            "job" => "Job",
            "partnership" => "Partnership",
            "research" => "Research",
            _ => "Opportunity",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OpportunitySavedSearch {
    pub opportunity_saved_search_id: Uuid,
    pub name: String,
    pub filters: Value,
    pub frequency: String,
    pub active: bool,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub next_run_at: Option<DateTime<Utc>>,
}

pub(crate) fn parse_tags(input: Option<&str>) -> Vec<String> {
    input
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .take(12)
        .map(|tag| tag.chars().take(MAX_LEN_TAG).collect())
        .collect()
}

fn application_url(value: &(impl AsRef<str> + ?Sized), _ctx: &()) -> garde::Result {
    let url = Url::parse(value.as_ref().trim())
        .map_err(|_| garde::Error::new("invalid application URL"))?;
    if matches!(url.scheme(), "http" | "https") {
        Ok(())
    } else {
        Err(garde::Error::new("application URL must use http or https"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_kinds_exclude_projected_sources() {
        let mut input = OpportunityInput {
            kind: "grant".into(),
            title: "Grant".into(),
            organization_name: "GOUP".into(),
            summary: "Summary".into(),
            description: "Description".into(),
            apply_url: "https://example.test".into(),
            location: None,
            remote: None,
            members_only: None,
            tags: None,
            opens_at: None,
            closes_at: None,
        };
        assert!(input.valid_kind());
        input.kind = "job".into();
        assert!(!input.valid_kind());
        input.kind = "cfs".into();
        assert!(!input.valid_kind());
    }

    #[test]
    fn saved_search_frequency_is_daily_or_weekly() {
        let mut input = OpportunitySavedSearchInput {
            name: "AI funding".into(),
            frequency: "daily".into(),
            query: Some("AI".into()),
            kind: Some("funding".into()),
            location: None,
            remote: Some(true),
        };
        assert!(input.valid_frequency());
        input.frequency = "weekly".into();
        assert!(input.valid_frequency());
        input.frequency = "hourly".into();
        assert!(!input.valid_frequency());
    }

    #[test]
    fn parse_tags_trims_drops_empty_and_limits_count() {
        let tags = parse_tags(Some(" Rust, , AI,Climate "));
        assert_eq!(tags, ["Rust", "AI", "Climate"]);
    }

    #[test]
    fn application_url_rejects_unsafe_schemes() {
        assert!(application_url("https://example.test/apply", &()).is_ok());
        assert!(application_url("javascript:alert(1)", &()).is_err());
        assert!(application_url("/relative", &()).is_err());
    }
}
