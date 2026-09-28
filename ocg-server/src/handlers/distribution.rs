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

fn is_frozen_key(key: &str) -> bool {
    FROZEN_KEYS.iter().any(|frozen| key.eq_ignore_ascii_case(frozen))
}

pub(crate) async fn redirect(
    State(db): State<DynDB>,
    Path(code): Path<String>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
) -> Result<impl IntoResponse, HandlerError> {
    let mut fingerprint = Sha256::new();
    fingerprint.update(code.as_bytes());
    fingerprint.update(chrono::Utc::now().date_naive().to_string().as_bytes());
    for name in [header::USER_AGENT, header::ACCEPT_LANGUAGE] {
        if let Some(value) = headers.get(name) {
            fingerprint.update(value.as_bytes());
        }
    }
    if let Some(value) = headers.get("x-forwarded-for") {
        fingerprint.update(value.as_bytes());
    }
    let fingerprint = hex::encode(fingerprint.finalize());
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
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(HandlerError::Deserialization(
            "redirect target must be an absolute HTTP(S) URL without credentials".into(),
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
    let mut pairs: Vec<(String, String)> = target
        .query_pairs()
        .filter(|(key, _)| !is_frozen_key(key))
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    if let Some(incoming) = incoming {
        let incoming_pairs: Vec<(String, String)> =
            serde_urlencoded::from_str(incoming).unwrap_or_default();
        pairs.extend(incoming_pairs.into_iter().filter(|(key, _)| !is_frozen_key(key)));
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
            target_url: "https://events.example/path?keep=one&utm_source=bad".into(),
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
        assert_eq!(query.iter().filter(|(k, _)| k == "keep").count(), 2);
    }

    #[test]
    fn unsafe_targets_are_rejected() {
        for target in [
            "javascript:alert(1)",
            "http://localhost/a",
            "http://127.0.0.1/a",
        ] {
            assert!(validate_redirect_target(target).is_err());
        }
    }
}
