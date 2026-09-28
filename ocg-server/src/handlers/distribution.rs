//! Privacy-preserving tracked distribution redirects.

use axum::{
    extract::{Path, RawQuery, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect},
};
use sha2::{Digest, Sha256};

use crate::{db::DynDB, handlers::error::HandlerError};

const FROZEN_KEYS: [&str; 6] = [
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_content",
    "referral_code",
    "ref",
];
const REDIRECT_KEYS: [&str; 9] = [
    "continue",
    "dest",
    "destination",
    "next",
    "redirect",
    "redirect_uri",
    "return",
    "return_to",
    "url",
];

fn is_frozen_key(key: &str) -> bool {
    FROZEN_KEYS.iter().any(|frozen| key.eq_ignore_ascii_case(frozen))
}

fn is_redirect_key(key: &str) -> bool {
    REDIRECT_KEYS
        .iter()
        .any(|redirect| key.eq_ignore_ascii_case(redirect))
}

fn fingerprint(code: &str, headers: &HeaderMap, day: chrono::NaiveDate) -> String {
    let mut fingerprint = Sha256::new();
    let day = day.to_string();
    let values = [
        Some(code.as_bytes()),
        Some(day.as_bytes()),
        headers.get(header::USER_AGENT).map(|value| value.as_bytes()),
        headers.get(header::ACCEPT_LANGUAGE).map(|value| value.as_bytes()),
        headers.get("x-forwarded-for").map(|value| value.as_bytes()),
    ];
    for value in values.into_iter().flatten() {
        fingerprint.update(value.len().to_be_bytes());
        fingerprint.update(value);
    }
    hex::encode(fingerprint.finalize())
}

pub(crate) async fn redirect(
    State(db): State<DynDB>,
    Path(code): Path<String>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
) -> Result<impl IntoResponse, HandlerError> {
    let fingerprint = fingerprint(&code, &headers, chrono::Utc::now().date_naive());
    let Some(link) = db.resolve_distribution_link(&code, &fingerprint).await? else {
        return Err(HandlerError::NotFound);
    };
    let target = merge_redirect_query(&link, query.as_deref())?;
    Ok((
        StatusCode::TEMPORARY_REDIRECT,
        [
            ("Cache-Control", "no-store"),
            ("Referrer-Policy", "no-referrer"),
        ],
        Redirect::temporary(target.as_str()),
    ))
}

pub(crate) fn validate_redirect_target(target: &str) -> Result<reqwest::Url, HandlerError> {
    let url = reqwest::Url::parse(target)
        .map_err(|error| HandlerError::Deserialization(error.to_string()))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url
            .query_pairs()
            .any(|(key, _)| is_frozen_key(&key) || is_redirect_key(&key))
    {
        return Err(HandlerError::Deserialization(
            "redirect target must be an absolute HTTPS URL without credentials, attribution, or nested redirect parameters".into(),
        ));
    }
    let host = url.host_str().unwrap_or_default();
    if host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || host.parse::<std::net::IpAddr>().is_ok_and(|ip| match ip {
            std::net::IpAddr::V4(ip) => {
                ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified()
            }
            std::net::IpAddr::V6(ip) => {
                ip.is_loopback() || ip.is_unspecified() || ip.is_unique_local()
            }
        })
    {
        return Err(HandlerError::Deserialization(
            "redirect target host is not public".into(),
        ));
    }
    Ok(url)
}

fn merge_redirect_query(
    link: &crate::types::distribution::ResolvedDistributionLink,
    incoming: Option<&str>,
) -> Result<reqwest::Url, HandlerError> {
    let mut target = validate_redirect_target(&link.target_url)?;
    let allowed_incoming_keys: Vec<String> = target
        .query_pairs()
        .filter(|(key, _)| !is_frozen_key(key))
        .map(|(key, _)| key.into_owned())
        .collect();
    let mut pairs: Vec<(String, String)> = target
        .query_pairs()
        .filter(|(key, _)| !is_frozen_key(key))
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    if let Some(incoming) = incoming {
        let incoming_pairs: Vec<(String, String)> =
            serde_urlencoded::from_str(incoming).unwrap_or_default();
        for (key, value) in incoming_pairs {
            if !is_frozen_key(&key)
                && !is_redirect_key(&key)
                && allowed_incoming_keys
                    .iter()
                    .any(|allowed| allowed.eq_ignore_ascii_case(&key))
            {
                pairs.retain(|(existing, _)| !existing.eq_ignore_ascii_case(&key));
                pairs.push((key, value));
            }
        }
    }
    pairs.extend([
        ("utm_source".into(), link.utm_source.clone()),
        ("utm_medium".into(), link.utm_medium.clone()),
        ("utm_campaign".into(), link.utm_campaign.clone()),
    ]);
    if let Some(value) = &link.utm_content {
        pairs.push(("utm_content".into(), value.clone()));
    }
    if let Some(value) = &link.referral_code {
        pairs.push(("referral_code".into(), value.clone()));
    }
    target.query_pairs_mut().clear().extend_pairs(pairs);
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::distribution::ResolvedDistributionLink;

    fn link() -> ResolvedDistributionLink {
        ResolvedDistributionLink {
            target_url: "https://events.example/path?keep=one".into(),
            utm_source: "linkedin".into(),
            utm_medium: "social".into(),
            utm_campaign: "launch".into(),
            utm_content: Some("speaker".into()),
            referral_code: Some("PARTNER".into()),
        }
    }

    #[test]
    fn frozen_values_replace_inbound_attribution() {
        let url = merge_redirect_query(
            &link(),
            Some("keep=two&UTM_CAMPAIGN=evil&referral_code=evil"),
        )
        .unwrap();
        let query: Vec<_> = url.query_pairs().collect();
        assert!(query.contains(&("utm_campaign".into(), "launch".into())));
        assert!(!query.contains(&("utm_campaign".into(), "evil".into())));
        assert_eq!(query.iter().filter(|(k, _)| k == "keep").count(), 1);
        assert!(query.contains(&("keep".into(), "two".into())));
    }

    #[test]
    fn unsafe_targets_are_rejected() {
        for target in [
            "javascript:alert(1)",
            "http://localhost/a",
            "http://127.0.0.1/a",
            "http://events.example/a",
            "https://events.example/a?next=https%3A%2F%2Fevil.example",
            "https://events.example/a?utm_source=mutable",
        ] {
            assert!(validate_redirect_target(target).is_err());
        }
    }

    #[test]
    fn incoming_query_is_limited_to_target_allowlist() {
        let url = merge_redirect_query(
            &link(),
            Some("keep=two&next=https%3A%2F%2Fevil.example&unknown=value"),
        )
        .unwrap();
        let query: Vec<_> = url.query_pairs().collect();
        assert!(query.contains(&("keep".into(), "two".into())));
        assert!(!query.iter().any(|(key, _)| key == "next"));
        assert!(!query.iter().any(|(key, _)| key == "unknown"));
    }

    #[test]
    fn fingerprint_is_day_scoped_and_unambiguous() {
        let day = chrono::NaiveDate::from_ymd_opt(2026, 9, 27).unwrap();
        let mut first = HeaderMap::new();
        first.insert(header::USER_AGENT, "ab".parse().unwrap());
        first.insert(header::ACCEPT_LANGUAGE, "c".parse().unwrap());
        let mut second = HeaderMap::new();
        second.insert(header::USER_AGENT, "a".parse().unwrap());
        second.insert(header::ACCEPT_LANGUAGE, "bc".parse().unwrap());

        assert_ne!(
            fingerprint("code", &first, day),
            fingerprint("code", &second, day)
        );
        assert_ne!(
            fingerprint("code", &first, day),
            fingerprint("code", &first, day.succ_opt().unwrap())
        );
    }
}
