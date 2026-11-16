//! Templates for dashboard pages.

/// Alliance dashboard templates.
pub mod alliance;
/// Shared dashboard audit log templates.
pub mod audit;
/// Upstream community-scoped templates adapted by alliance handlers.
pub mod community;
/// Group dashboard templates.
pub mod group;
/// Shared GTM dashboard templates.
pub mod gtm;
/// Shared inbox templates.
pub mod inbox;
/// Jobs dashboard templates.
pub mod jobs;
/// Opportunity organizer dashboard templates.
pub mod opportunities;
/// User dashboard templates.
pub mod user;

pub use ocg_core::types::pagination::{DASHBOARD_PAGINATION_LIMIT, default_limit, default_offset};
