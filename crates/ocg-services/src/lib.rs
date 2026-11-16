//! Services modules.

/// Activity tracker service module.
pub mod activity_tracker;
/// Custom hostname DNS verification.
pub mod custom_domains;
/// Scheduled external event discovery.
pub mod event_discovery;
/// Draft-only GTM agents for alliance and group leads.
pub mod gtm;
/// Images service module.
pub mod images;
/// Scheduled global jobs discovery.
pub mod job_discovery;

/// Group inbox service module.
pub mod inbox;

/// Meetings service module.
pub mod meetings;

/// Notifications service module.
pub mod notifications;

/// Payments service module.
pub mod payments;

/// Redis Pub/Sub-backed realtime delivery.
pub mod realtime;
/// Recording publishing service module.
pub mod recording_publishing;

/// Shared background worker helpers.
pub mod workers;
