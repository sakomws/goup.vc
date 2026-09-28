//! Templates for the user dashboard invitations tab.

use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    templates::helpers::DATE_FORMAT_2,
    types::{alliance::AllianceRole, group::GroupRole},
};

// Pages templates.

/// List page showing pending invitations for the user.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/user/invitations_list.html")]
pub(crate) struct ListPage {
    /// Pending alliance invitations for the current user.
    pub alliance_invitations: Vec<AllianceTeamInvitation>,
    /// Pending event invitations for the current user.
    pub event_invitations: Vec<EventInvitation>,
    /// Pending co-host invitations the current user can decide.
    pub event_cohost_invitations: Vec<EventCohostInvitation>,
    /// Pending group invitations for the current user.
    pub group_invitations: Vec<GroupTeamInvitation>,
}

impl ListPage {
    /// Returns the total number of pending invitations shown on the page.
    pub(crate) fn total_invitations(&self) -> i64 {
        let total = self.alliance_invitations.len()
            + self.event_invitations.len()
            + self.event_cohost_invitations.len()
            + self.group_invitations.len();
        i64::try_from(total).expect("invitation count to fit in i64")
    }
}

// Types.

/// Pending event co-host invitation available to an eligible organizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventCohostInvitation {
    /// Invited peer group identifier.
    pub cohost_group_id: Uuid,
    /// Invited peer group display name.
    pub cohost_group_name: String,
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
    /// Optional note from the inviting organizer.
    pub message: Option<String>,
    /// Primary alliance public name.
    pub primary_alliance_name: String,
    /// Primary host group identifier.
    pub primary_group_id: Uuid,
    /// Primary host group display name.
    pub primary_group_name: String,
    /// Primary host group public slug.
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

/// Alliance team invitation summary information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AllianceTeamInvitation {
    /// Alliance identifier.
    pub alliance_id: Uuid,
    /// Alliance name (slug).
    pub alliance_name: String,
    /// Role within the alliance.
    pub role: AllianceRole,

    /// Invitation creation time.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

/// Organizer-created event invitation summary information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventInvitation {
    /// Human-readable display name of the alliance.
    pub alliance_display_name: String,
    /// Alliance slug.
    pub alliance_name: String,
    /// Event identifier.
    pub event_id: Uuid,
    /// Event display name.
    pub event_name: String,
    /// Group display name.
    pub group_name: String,
    /// Timezone in which event dates should be displayed.
    pub timezone: chrono_tz::Tz,

    /// Invitation creation time.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    /// Event start time.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub starts_at: Option<DateTime<Utc>>,
}

/// Group team invitation summary information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GroupTeamInvitation {
    /// Alliance name (slug).
    pub alliance_name: String,
    /// Group identifier.
    pub group_id: Uuid,
    /// Group name.
    pub group_name: String,
    /// Role within the group.
    pub role: GroupRole,

    /// Invitation creation time.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}
