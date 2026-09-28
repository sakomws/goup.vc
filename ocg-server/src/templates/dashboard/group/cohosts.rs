//! Templates and types for event co-hosting in the group dashboard.

use askama::Template;
use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::validation::{MAX_LEN_L, trimmed_non_empty_opt};

/// Co-host invitation inbox for the selected group.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/group/cohosts_list.html")]
pub(crate) struct ListPage {
    /// Invitations awaiting a decision from this group.
    pub invitations: Vec<EventCohostInvitation>,
}

/// Co-host controls for an event owned by the selected group.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/group/event_cohosts.html")]
pub(crate) struct EventCohostsPage {
    /// Groups that can be invited to co-host.
    pub candidates: Vec<EventCohostCandidate>,
    /// Event identifier.
    pub event_id: Uuid,
    /// Active and historical invitations for this event.
    pub requests: Vec<EventCohostRequest>,
}

/// A group that can be invited to co-host an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventCohostCandidate {
    /// Alliance identifier.
    pub alliance_id: Uuid,
    /// Alliance public name.
    pub alliance_name: String,
    /// Distance from the primary event, when both locations are known.
    pub distance_km: Option<f64>,
    /// Group identifier.
    pub group_id: Uuid,
    /// Group display name.
    pub group_name: String,
    /// Group public slug.
    pub group_slug: String,
    /// Whether this group belongs to the event owner's alliance.
    pub same_alliance: bool,
}

/// Display data used to compose a co-host invitation notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventCohostNotificationData {
    /// Invited peer group display name.
    pub cohost_group_name: String,
    /// Event display name.
    pub event_name: String,
    /// Optional note supplied by the inviting organizer.
    pub message: Option<String>,
    /// Primary host group display name.
    pub primary_group_name: String,
}

/// A pending co-host invitation shown to the invited group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventCohostInvitation {
    /// Co-host request identifier.
    pub event_cohost_id: Uuid,
    /// Event identifier.
    pub event_id: Uuid,
    /// Event display name.
    pub event_name: String,
    /// Event public slug.
    pub event_slug: String,
    /// Event start time, when scheduled.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub event_starts_at: Option<DateTime<Utc>>,
    /// Optional note from the inviting group.
    pub message: Option<String>,
    /// Primary alliance public name.
    pub primary_alliance_name: String,
    /// Primary group identifier.
    pub primary_group_id: Uuid,
    /// Primary group display name.
    pub primary_group_name: String,
    /// Primary group public slug.
    pub primary_group_slug: String,
    /// Invitation creation time.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub requested_at: DateTime<Utc>,
}

impl EventCohostInvitation {
    /// Returns the canonical public event URL.
    pub(crate) fn event_url(&self) -> String {
        format!(
            "/{}/group/{}/event/{}",
            self.primary_alliance_name, self.primary_group_slug, self.event_slug
        )
    }
}

/// A co-host request shown to the primary event owner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventCohostRequest {
    /// Co-host request identifier.
    pub event_cohost_id: Uuid,
    /// Invited group identifier.
    pub cohost_group_id: Uuid,
    /// Invited alliance public name.
    pub alliance_name: String,
    /// Invited group display name.
    pub group_name: String,
    /// Invited group public slug.
    pub group_slug: String,
    /// Optional invitation note.
    pub message: Option<String>,
    /// Invitation creation time.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub requested_at: DateTime<Utc>,
    /// Current invitation status.
    pub status: String,
}

/// Form used by an owning group to invite another group.
#[derive(Debug, Clone, Deserialize, Validate)]
pub(crate) struct RequestEventCohost {
    /// Group to invite.
    #[garde(skip)]
    pub cohost_group_id: Uuid,
    /// Optional message for the invited group's organizers.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_L))]
    pub message: Option<String>,
}
