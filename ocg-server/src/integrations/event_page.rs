//! Safe extraction of structured event metadata from approved discovery sources.

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use reqwest::{Client, Url, redirect};
use serde_json::Value;

use crate::integrations::you_com::{DiscoveredEvent, source_search_domain};

const MAX_HTML_BYTES: usize = 1_000_000;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// Fetches structured event metadata without following untrusted redirects.
#[derive(Clone)]
pub(crate) struct EventPageClient {
    http: Client,
}

impl EventPageClient {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            http: Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .redirect(redirect::Policy::none())
                .user_agent("GOUP-event-discovery/1.0")
                .build()?,
        })
    }

    /// Extracts an Event JSON-LD record from a candidate page.
    pub(crate) async fn fetch(
        &self,
        candidate_url: &str,
        approved_source_url: &str,
    ) -> Result<Option<DiscoveredEvent>> {
        let Some(html) = self.fetch_html(candidate_url, approved_source_url).await? else {
            return Ok(None);
        };
        Ok(extract_event_json_ld(&html, candidate_url))
    }

    /// Reads the public event cards from You.com's official events landing page.
    pub(crate) async fn discover_source_events(
        &self,
        source_url: &str,
        city: &str,
    ) -> Result<Vec<DiscoveredEvent>> {
        if !is_you_events_source(source_url) {
            return Ok(vec![]);
        }
        let Some(html) = self.fetch_html(source_url, source_url).await? else {
            return Ok(vec![]);
        };
        Ok(extract_you_com_events(&html, city))
    }

    /// Fetches an approved candidate page for a structured-data extractor.
    pub(crate) async fn fetch_html(
        &self,
        candidate_url: &str,
        approved_source_url: &str,
    ) -> Result<Option<String>> {
        validate_candidate_url(candidate_url, approved_source_url).await?;
        let response = self
            .http
            .get(candidate_url)
            .send()
            .await
            .context("requesting event candidate page")?
            .error_for_status()
            .context("event candidate page returned an error")?;

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        if !content_type.starts_with("text/html") {
            return Ok(None);
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_HTML_BYTES as u64)
        {
            return Ok(None);
        }

        let mut body = Vec::new();
        let mut response = response;
        while let Some(chunk) = response.chunk().await.context("reading event candidate page")? {
            if body.len() + chunk.len() > MAX_HTML_BYTES {
                return Ok(None);
            }
            body.extend_from_slice(&chunk);
        }
        let html = String::from_utf8(body).context("event candidate page was not UTF-8")?;
        Ok(Some(html))
    }
}

fn is_you_events_source(source_url: &str) -> bool {
    let Ok(url) = Url::parse(source_url) else {
        return false;
    };
    matches!(url.host_str(), Some("you.com") | Some("www.you.com"))
        && url.path().trim_end_matches('/') == "/events"
}

#[derive(Debug, PartialEq, Eq)]
struct SourceEventCard {
    title: String,
    location: String,
    starts_at: DateTime<Utc>,
    source_url: String,
}

/// Extracts current cards from the public Webflow event listing.
///
/// These cards list a date but not a start time. We represent the date as noon Pacific
/// so it remains on the advertised local day; organizers review the draft before publishing.
pub(crate) fn extract_you_com_events(html: &str, city: &str) -> Vec<DiscoveredEvent> {
    let mut remaining = html;
    let mut events = Vec::new();
    while let Some(start) = remaining.find("events3_item") {
        let card = &remaining[start..];
        let next = card["events3_item".len()..]
            .find("events3_item")
            .map(|offset| offset + "events3_item".len());
        let block = &card[..next.unwrap_or(card.len())];
        remaining = &card[block.len()..];

        let Some(url) = html_attribute(block, "href") else {
            continue;
        };
        let Some(title) = html_class_text(block, "events3_card_heading") else {
            continue;
        };
        let labels = html_class_texts(block, "events2_card_label");
        let (Some(date), Some(location)) = (labels.first(), labels.last()) else {
            continue;
        };
        let Ok(date) = NaiveDate::parse_from_str(date, "%B %d, %Y") else {
            continue;
        };
        let Some(starts_at) = chrono_tz::US::Pacific
            .with_ymd_and_hms(date.year(), date.month(), date.day(), 12, 0, 0)
            .single()
            .map(|time| time.with_timezone(&Utc))
        else {
            continue;
        };
        let card = SourceEventCard {
            title,
            location: location.to_owned(),
            starts_at,
            source_url: url.to_owned(),
        };
        if city_matches_event(&card.location, city) && card.starts_at > Utc::now() {
            events.push(DiscoveredEvent {
                title: card.title,
                description: Some(format!("Official You.com event · {}", card.location)),
                starts_at: card.starts_at,
                source_url: card.source_url,
            });
        }
    }
    events
}

fn city_matches_event(location: &str, city: &str) -> bool {
    location.eq_ignore_ascii_case("remote")
        || location
            .to_ascii_lowercase()
            .contains(&city.trim().to_ascii_lowercase())
}

fn html_attribute<'a>(html: &'a str, attribute: &str) -> Option<&'a str> {
    let prefix = format!("{attribute}=\"");
    let value = html.split_once(&prefix)?.1;
    value.split_once('"').map(|(value, _)| value)
}

fn html_class_text(html: &str, class: &str) -> Option<String> {
    html_class_texts(html, class).into_iter().next()
}

fn html_class_texts(html: &str, class: &str) -> Vec<String> {
    let marker = format!("class=\"{class}");
    let mut remaining = html;
    let mut texts = Vec::new();
    while let Some(before) = remaining.split_once(&marker) {
        let after_class = before.1;
        let Some((_, content)) = after_class.split_once('>') else {
            break;
        };
        let Some((text, rest)) = content.split_once('<') else {
            break;
        };
        let text = text.trim();
        if !text.is_empty() {
            texts.push(text.to_owned());
        }
        remaining = rest;
    }
    texts
}

/// Checks that a candidate remains within the organizer-approved public domain.
pub(crate) async fn validate_candidate_url(
    candidate_url: &str,
    approved_source_url: &str,
) -> Result<()> {
    let candidate = Url::parse(candidate_url).context("event candidate URL must be absolute")?;
    if !matches!(candidate.scheme(), "http" | "https") {
        bail!("event candidate URL must use HTTP or HTTPS");
    }

    let allowed_domain = source_search_domain(approved_source_url)?;
    let candidate_host = candidate
        .host_str()
        .context("event candidate URL must include a host")?
        .to_ascii_lowercase();
    let allowed_domain = allowed_domain.to_ascii_lowercase();
    if !is_approved_host(&candidate_host, &allowed_domain) {
        bail!("event candidate URL is outside the approved source domain");
    }

    let port = candidate.port_or_known_default().unwrap_or(443);
    let addresses: Vec<IpAddr> = tokio::net::lookup_host((candidate_host.as_str(), port))
        .await
        .context("resolving event candidate host")?
        .map(|address| address.ip())
        .collect();
    if addresses.is_empty() || addresses.iter().any(|address| !is_public_address(*address)) {
        bail!("event candidate URL resolves to a non-public address");
    }
    Ok(())
}

fn is_approved_host(candidate_host: &str, approved_host: &str) -> bool {
    candidate_host == approved_host
        || candidate_host.ends_with(&format!(".{approved_host}"))
        || approved_host
            .strip_prefix("www.")
            .is_some_and(|apex| candidate_host == apex)
        || candidate_host
            .strip_prefix("www.")
            .is_some_and(|apex| apex == approved_host)
}

fn is_public_address(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => {
            !address.is_private()
                && !address.is_loopback()
                && !address.is_link_local()
                && !address.is_broadcast()
                && !address.is_unspecified()
                && !is_documentation_v4(address)
                && address != Ipv4Addr::new(169, 254, 169, 254)
        }
        IpAddr::V6(address) => {
            !address.is_loopback()
                && !address.is_unspecified()
                && !address.is_unicast_link_local()
                && !address.is_unique_local()
                && !is_documentation_v6(address)
                && address != Ipv6Addr::LOCALHOST
        }
    }
}

fn is_documentation_v4(address: Ipv4Addr) -> bool {
    matches!(
        address.octets(),
        [192, 0, 2, _] | [198, 51, 100, _] | [203, 0, 113, _]
    )
}

fn is_documentation_v6(address: Ipv6Addr) -> bool {
    address.segments()[..2] == [0x2001, 0x0db8]
}

/// Reads schema.org Event JSON-LD scripts from an HTML page.
pub(crate) fn extract_event_json_ld(html: &str, candidate_url: &str) -> Option<DiscoveredEvent> {
    json_ld_scripts(html)
        .filter_map(|json| serde_json::from_str::<Value>(json).ok())
        .find_map(|value| event_from_json_ld(&value, candidate_url))
}

pub(crate) fn json_ld_scripts(html: &str) -> impl Iterator<Item = &str> {
    let mut remaining = html;
    std::iter::from_fn(move || {
        loop {
            let start = remaining.find("<script")?;
            let after_start = &remaining[start + "<script".len()..];
            let tag_end = after_start.find('>')?;
            let attributes = after_start[..tag_end].to_ascii_lowercase();
            let content = &after_start[tag_end + 1..];
            let end = content.find("</script>")?;
            remaining = &content[end + "</script>".len()..];
            if attributes.contains("type=\"application/ld+json\"")
                || attributes.contains("type='application/ld+json'")
            {
                return Some(&content[..end]);
            }
        }
    })
}

fn event_from_json_ld(value: &Value, candidate_url: &str) -> Option<DiscoveredEvent> {
    match value {
        Value::Array(values) => values
            .iter()
            .find_map(|value| event_from_json_ld(value, candidate_url)),
        Value::Object(object) => {
            if is_event(object.get("@type")) {
                return event_from_object(object, candidate_url);
            }
            object
                .get("@graph")
                .and_then(|graph| event_from_json_ld(graph, candidate_url))
        }
        _ => None,
    }
}

fn is_event(value: Option<&Value>) -> bool {
    match value {
        Some(Value::String(kind)) => kind.eq_ignore_ascii_case("Event"),
        Some(Value::Array(kinds)) => kinds
            .iter()
            .any(|kind| kind.as_str().is_some_and(|kind| kind.eq_ignore_ascii_case("Event"))),
        _ => false,
    }
}

fn event_from_object(
    object: &serde_json::Map<String, Value>,
    candidate_url: &str,
) -> Option<DiscoveredEvent> {
    let title = object.get("name")?.as_str()?.trim();
    if title.is_empty() || title.len() > 240 {
        return None;
    }
    let starts_at = DateTime::parse_from_rfc3339(object.get("startDate")?.as_str()?)
        .ok()?
        .with_timezone(&Utc);
    if starts_at <= Utc::now() {
        return None;
    }
    let description = object
        .get("description")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|description| !description.is_empty())
        .map(str::to_owned);

    Some(DiscoveredEvent {
        title: title.to_owned(),
        description,
        starts_at,
        source_url: candidate_url.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_event_from_json_ld_graph() {
        let event = extract_event_json_ld(
            r#"<script type="application/ld+json">
                {"@graph":[{"@type":"Event","name":"Baku Rust meetup","description":"Talks and networking.","startDate":"2099-07-21T19:00:00+04:00"}]}
            </script>"#,
            "https://events.example.com/rust-meetup",
        )
        .unwrap();

        assert_eq!(event.title, "Baku Rust meetup");
        assert_eq!(event.source_url, "https://events.example.com/rust-meetup");
    }

    #[test]
    fn rejects_event_without_a_timezone() {
        assert!(
            extract_event_json_ld(
                r#"<script type="application/ld+json">{"@type":"Event","name":"Baku meetup","startDate":"2099-07-21T19:00:00"}</script>"#,
                "https://events.example.com/meetup",
            )
            .is_none()
        );
    }

    #[test]
    fn ignores_non_event_json_ld() {
        assert!(
            extract_event_json_ld(
                r#"<script type="application/ld+json">{"@type":"Organization","name":"GOUP"}</script>"#,
                "https://events.example.com",
            )
            .is_none()
        );
    }

    #[test]
    fn rejects_private_addresses() {
        assert!(!is_public_address(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
        assert!(!is_public_address(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(!is_public_address(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }

    #[test]
    fn accepts_apex_and_www_equivalents() {
        assert!(is_approved_host("example.com", "www.example.com"));
        assert!(is_approved_host("www.example.com", "example.com"));
        assert!(is_approved_host("events.example.com", "example.com"));
        assert!(!is_approved_host("events.example.com", "www.example.com"));
    }

    #[test]
    fn extracts_you_com_event_cards_for_city_and_remote() {
        let events = extract_you_com_events(
            r#"
            <div class="events3_item"><a href="https://luma.com/sf"></a>
              <div class="events2_card_label">July 30, 2099</div>
              <div class="events2_card_label">|</div>
              <div class="events2_card_label">San Francisco</div>
              <h2 class="events3_card_heading">SF Hackathon</h2>
            </div>
            <div class="events3_item"><a href="https://luma.com/remote"></a>
              <div class="events2_card_label">July 31, 2099</div>
              <div class="events2_card_label">|</div>
              <div class="events2_card_label">Remote</div>
              <h2 class="events3_card_heading">Virtual workshop</h2>
            </div>
            "#,
            "Baku",
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Virtual workshop");
        assert_eq!(events[0].source_url, "https://luma.com/remote");
    }
}
