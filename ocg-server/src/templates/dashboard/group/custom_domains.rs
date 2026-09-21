//! Templates and forms for group and event custom hostnames.

use askama::Template;
use garde::Validate;
use serde::{Deserialize, Deserializer};

use crate::{
    types::custom_domain::CustomDomain,
    validation::{MAX_LEN_CUSTOM_DOMAIN, custom_domain},
};

/// Reusable custom-domain management card.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/group/custom_domain_card.html")]
pub(crate) struct Card {
    /// Base dashboard endpoint for this target.
    pub action_url: String,
    /// Whether the current user may manage this domain.
    pub can_manage: bool,
    /// Current custom-domain assignment.
    pub domain: Option<CustomDomain>,
    /// Human-readable target kind.
    pub target_label: &'static str,
}

/// Hostname form payload.
#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct CustomDomainInput {
    /// Hostname without a scheme or path.
    #[serde(deserialize_with = "deserialize_hostname")]
    #[garde(custom(custom_domain), length(max = MAX_LEN_CUSTOM_DOMAIN))]
    pub hostname: String,
}

fn deserialize_hostname<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).map(|hostname| {
        let hostname = hostname.trim().trim_end_matches('.');
        if hostname.contains(['/', ':', '*']) {
            return hostname.to_ascii_lowercase();
        }
        reqwest::Url::parse(&format!("https://{hostname}"))
            .ok()
            .and_then(|url| url.host_str().map(str::to_string))
            .unwrap_or_else(|| hostname.to_ascii_lowercase())
    })
}
