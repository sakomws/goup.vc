//! You.com search integration used to discover Baku community events.

use std::{collections::HashSet, time::Duration};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::YouComConfig;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// A normalized community event ready for persistence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DiscoveredEvent {
    pub title: String,
    pub description: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub source_url: String,
}

impl DiscoveredEvent {
    /// A stable deduplication key scoped to a group.
    pub(crate) fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.title.trim().to_lowercase());
        hasher.update(b"\0");
        hasher.update(self.starts_at.timestamp().to_le_bytes());
        hasher.update(b"\0");
        hasher.update(self.source_url.trim().to_lowercase());
        hex::encode(hasher.finalize())
    }
}

/// Thin, typed client for the You.com search API.
#[derive(Clone)]
pub(crate) struct YouComClient {
    api_key: String,
    http: Client,
    search_url: String,
}

impl YouComClient {
    pub(crate) fn new(cfg: &YouComConfig) -> Result<Self> {
        let http = Client::builder().timeout(REQUEST_TIMEOUT).build()?;
        Ok(Self {
            api_key: cfg.api_key.clone(),
            http,
            search_url: cfg.search_url.clone(),
        })
    }

    /// Searches You.com with a caller-provided, source-scoped query.
    pub(crate) async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let mut search_url =
            reqwest::Url::parse(&self.search_url).context("invalid You.com search URL")?;
        search_url.query_pairs_mut().append_pair("query", query);
        let response = self
            .http
            .get(search_url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("requesting You.com search")?
            .error_for_status()
            .context("You.com search returned an error")?;
        let payload: SearchResponse =
            response.json().await.context("decoding You.com search response")?;
        Ok(payload.results.web.into_iter().chain(payload.results.news).collect())
    }
}

/// Search result fields shared by the supported You.com response shapes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SearchResult {
    #[serde(alias = "name")]
    pub title: String,
    #[serde(alias = "url", alias = "link")]
    pub url: String,
    #[serde(default, alias = "snippet", alias = "description")]
    pub description: Option<String>,
    #[serde(default)]
    pub snippets: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    results: SearchResults,
}

#[derive(Debug, Default, Deserialize)]
struct SearchResults {
    #[serde(default)]
    web: Vec<SearchResult>,
    #[serde(default)]
    news: Vec<SearchResult>,
}

/// Rejects pages that do not identify the group's configured city.
pub(crate) fn is_city_relevant(result: &SearchResult, city: &str) -> bool {
    let city = city.trim().to_lowercase();
    if city.is_empty() {
        return false;
    }
    let text = format!(
        "{} {} {} {}",
        result.title,
        result.description.as_deref().unwrap_or_default(),
        result.snippets.join(" "),
        result.url
    )
    .to_lowercase();
    text.contains(&city)
}

/// Returns unique results relevant to the configured city while preserving discovery order.
pub(crate) fn unique_city_results(results: Vec<SearchResult>, city: &str) -> Vec<SearchResult> {
    let mut urls = HashSet::new();
    results
        .into_iter()
        .filter(|result| is_city_relevant(result, city))
        .filter(|result| urls.insert(result.url.trim().to_lowercase()))
        .collect()
}

/// Validates a source URL entered in a group dashboard.
pub(crate) fn validate_source_url(url: &str) -> Result<()> {
    source_search_domain(url).map(|_| ())
}

/// Returns the hostname suitable for a You.com `site:` search operator.
///
/// Dashboard sources are full URLs, while `site:` accepts only a domain.
pub(crate) fn source_search_domain(url: &str) -> Result<String> {
    let parsed = reqwest::Url::parse(url).context("source URL must be absolute")?;
    if !matches!(parsed.scheme(), "http" | "https") {
        bail!("source URL must use HTTP or HTTPS");
    }
    parsed
        .host_str()
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("source URL must include a host"))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn result(title: &str, url: &str, description: Option<&str>) -> SearchResult {
        SearchResult {
            title: title.into(),
            url: url.into(),
            description: description.map(str::to_owned),
            snippets: vec![],
        }
    }

    #[test]
    fn retains_unique_results_for_configured_city() {
        let results = unique_city_results(
            vec![
                result(
                    "San Francisco Rust meetup",
                    "https://example.test/events/1",
                    None,
                ),
                result(
                    "San Francisco Rust meetup",
                    "https://example.test/events/1",
                    None,
                ),
                result("Tbilisi meetup", "https://example.test/events/2", None),
            ],
            "San Francisco",
        );
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn extracts_search_domain_from_source_url() {
        assert_eq!(
            source_search_domain("https://careers.example.com/open-roles?team=engineering")
                .unwrap(),
            "careers.example.com"
        );
    }

    #[test]
    fn fingerprint_changes_when_event_time_changes() {
        let event = DiscoveredEvent {
            title: "Baku meetup".into(),
            description: None,
            starts_at: Utc.with_ymd_and_hms(2026, 7, 21, 12, 0, 0).unwrap(),
            source_url: "https://example.test/events/1".into(),
        };
        let mut rescheduled = event.clone();
        rescheduled.starts_at = Utc.with_ymd_and_hms(2026, 7, 22, 12, 0, 0).unwrap();
        assert_ne!(event.fingerprint(), rescheduled.fingerprint());
    }

    #[test]
    fn decodes_web_and_news_results() {
        let payload: SearchResponse = serde_json::from_str(
            r#"{
                "results": {
                    "web": [{"title": "Baku meetup", "url": "https://example.test/web"}],
                    "news": [{"title": "Baku news", "url": "https://example.test/news"}]
                }
            }"#,
        )
        .unwrap();

        assert_eq!(payload.results.web.len(), 1);
        assert_eq!(payload.results.news.len(), 1);
    }
}
