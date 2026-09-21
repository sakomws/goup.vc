//! Custom hostname types shared by dashboard management and public routing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A custom hostname assigned to one group or event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CustomDomain {
    /// Time when an operator activated TLS and routing.
    pub activated_at: Option<DateTime<Utc>>,
    /// Stable identifier used to bind DNS verification to this exact assignment.
    pub custom_domain_id: Uuid,
    /// Event target, when this is an event hostname.
    pub event_id: Option<Uuid>,
    /// Group target, when this is a group hostname.
    pub group_id: Option<Uuid>,
    /// Normalized hostname without scheme, path, or port.
    pub hostname: String,
    /// Time when DNS ownership was verified.
    pub verified_at: Option<DateTime<Utc>>,
    /// Value expected in the DNS verification TXT record.
    pub verification_token: String,
}

impl CustomDomain {
    /// DNS TXT record name used to verify control of this hostname.
    pub(crate) fn verification_record_name(&self) -> String {
        format!("_goup-verification.{}", self.hostname)
    }

    /// Human-readable provisioning status.
    pub(crate) fn status(&self) -> &'static str {
        if self.activated_at.is_some() {
            "Active"
        } else if self.verified_at.is_some() {
            "Verified — awaiting TLS activation"
        } else {
            "Pending DNS verification"
        }
    }
}

/// Active custom-domain target used by host routing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CustomDomainTarget {
    /// Alliance path segment.
    pub alliance_name: String,
    /// Event path segment for event domains.
    pub event_slug: Option<String>,
    /// Public group path segment.
    pub group_slug: String,
    /// Active custom hostname.
    pub hostname: String,
}

impl CustomDomainTarget {
    /// Canonical GOUP path backing this hostname.
    pub(crate) fn canonical_path(&self) -> String {
        if let Some(event_slug) = &self.event_slug {
            format!(
                "/{}/group/{}/event/{event_slug}",
                self.alliance_name, self.group_slug
            )
        } else {
            format!("/{}/group/{}", self.alliance_name, self.group_slug)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_domain_uses_group_path() {
        let target = CustomDomainTarget {
            alliance_name: "goup".to_string(),
            event_slug: None,
            group_slug: "sf".to_string(),
            hostname: "sf.example.com".to_string(),
        };

        assert_eq!(target.canonical_path(), "/goup/group/sf");
    }

    #[test]
    fn test_event_domain_uses_event_path() {
        let target = CustomDomainTarget {
            alliance_name: "goup".to_string(),
            event_slug: Some("ai-forum".to_string()),
            group_slug: "sf".to_string(),
            hostname: "forum.example.com".to_string(),
        };

        assert_eq!(target.canonical_path(), "/goup/group/sf/event/ai-forum");
    }
}
