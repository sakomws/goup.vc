//! Types for planning and measuring group distribution.

use chrono::{DateTime, NaiveDateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::validation::{MAX_LEN_L, MAX_LEN_M, optional_trimmed_string, trimmed_non_empty};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DistributionDashboard {
    pub events: Vec<DistributionEvent>,
    pub campaigns: Vec<DistributionCampaign>,
    pub partners: Vec<DistributionPartner>,
    pub links: Vec<DistributionLink>,
    pub content: Vec<DistributionContent>,
    pub library: Vec<DistributionLibraryItem>,
    pub channel_metrics: Vec<DistributionChannelMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DistributionEvent {
    pub event_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DistributionCampaign {
    pub distribution_campaign_id: Uuid,
    pub event_id: Option<Uuid>,
    pub name: String,
    pub status: String,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub starts_at: Option<DateTime<Utc>>,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub ends_at: Option<DateTime<Utc>>,
    pub clicks: i64,
    pub registrations: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DistributionPartner {
    pub distribution_partner_id: Uuid,
    pub name: String,
    pub referral_code: String,
    pub active: bool,
    pub clicks: i64,
    pub registrations: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DistributionLink {
    pub distribution_link_id: Uuid,
    pub distribution_campaign_id: Uuid,
    pub distribution_partner_id: Option<Uuid>,
    pub code: String,
    pub channel: String,
    pub target_url: String,
    pub utm_source: String,
    pub utm_medium: String,
    pub utm_campaign: String,
    pub utm_content: Option<String>,
    pub active: bool,
    pub clicks: i64,
    pub registrations: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DistributionContent {
    pub distribution_content_id: Uuid,
    pub distribution_campaign_id: Uuid,
    pub channel: String,
    pub state: String,
    pub title: String,
    pub caption: String,
    pub cta: String,
    pub hashtags: String,
    pub event_image_reference: Option<String>,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub scheduled_for: Option<DateTime<Utc>>,
    pub remind_when_due: bool,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub posted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DistributionLibraryItem {
    pub distribution_library_item_id: Uuid,
    pub kind: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DistributionChannelMetric {
    pub channel: String,
    pub clicks: i64,
    pub registrations: i64,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct CampaignInput {
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub name: String,
    #[garde(skip)]
    #[serde(default, deserialize_with = "optional_uuid")]
    pub event_id: Option<Uuid>,
    #[garde(skip)]
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct PartnerInput {
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub name: String,
    #[garde(pattern(r"^[A-Za-z0-9_-]{2,64}$"))]
    pub referral_code: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct LinkInput {
    #[garde(skip)]
    pub distribution_campaign_id: Uuid,
    #[garde(skip)]
    #[serde(default, deserialize_with = "optional_uuid")]
    pub distribution_partner_id: Option<Uuid>,
    #[garde(pattern(r"^[A-Za-z0-9_-]{3,64}$"))]
    pub code: String,
    #[garde(skip)]
    pub channel: String,
    #[garde(url, length(max = MAX_LEN_L))]
    pub target_url: String,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub utm_source: String,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub utm_medium: String,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub utm_campaign: String,
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(length(max = MAX_LEN_M))]
    pub utm_content: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct ContentInput {
    #[garde(skip)]
    pub distribution_campaign_id: Uuid,
    #[garde(skip)]
    pub channel: String,
    #[garde(skip)]
    pub state: String,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub title: String,
    #[garde(length(max = MAX_LEN_L))]
    pub caption: String,
    #[garde(length(max = MAX_LEN_M))]
    pub cta: String,
    #[garde(length(max = MAX_LEN_M))]
    pub hashtags: String,
    #[garde(length(max = MAX_LEN_L))]
    pub event_image_reference: Option<String>,
    #[serde(default, deserialize_with = "optional_trimmed_string")]
    #[garde(length(max = MAX_LEN_M), custom(valid_scheduled_for))]
    pub scheduled_for: Option<String>,
    #[serde(default)]
    #[garde(skip)]
    pub remind_when_due: bool,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct LibraryInput {
    #[garde(skip)]
    pub kind: String,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub name: String,
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_L))]
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct ContentStateInput {
    #[garde(skip)]
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ResolvedDistributionLink {
    pub target_url: String,
    pub utm_source: String,
    pub utm_medium: String,
    pub utm_campaign: String,
    pub utm_content: Option<String>,
    pub referral_code: Option<String>,
}

fn optional_uuid<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    value
        .filter(|value| !value.trim().is_empty())
        .map(|value| Uuid::parse_str(value.trim()).map_err(serde::de::Error::custom))
        .transpose()
}

fn valid_scheduled_for(value: &Option<String>, _ctx: &()) -> garde::Result {
    let Some(value) = value else {
        return Ok(());
    };
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S"))
        .map(|_| ())
        .map_err(|_| garde::Error::new("invalid UTC schedule"))
}
