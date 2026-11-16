//! Templates for the group dashboard.

use serde::{Deserialize, Serialize};

pub mod accelerator;
pub mod analytics;
pub mod attendees;
pub mod book_exchange;
pub mod check_in;
pub mod coffee_meet;
pub mod cohosts;
pub mod custom_domains;
pub mod distribution;
pub mod event_analytics;
pub mod events;
pub mod home;
pub mod integrations;
pub mod intentional_dating;
pub mod invitation_requests;
pub mod members;
pub mod projects;
pub mod refunds;
pub mod rolling_cfs;
pub mod settings;
pub mod sponsors;
pub mod spotlights;
pub mod store;
pub mod submissions;
pub mod team;
pub mod waitlist;

/// Presence filter for optional database fields.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumString,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum PresenceFilter {
    /// Field value must be missing.
    Missing,
    /// Field value must be present.
    Present,
}
