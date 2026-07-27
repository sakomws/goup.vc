//! Services modules.

/// Scheduled external event discovery.
pub(crate) mod event_discovery;
/// Images service module.
pub(crate) mod images;
/// Scheduled global jobs discovery.
pub(crate) mod job_discovery;

/// Meetings service module.
pub(crate) mod meetings;

/// Notifications service module.
pub(crate) mod notifications;

/// Payments service module.
pub(crate) mod payments;

/// Redis Pub/Sub-backed realtime delivery.
pub(crate) mod realtime;
/// Recording publishing service module.
pub(crate) mod recording_publishing;
