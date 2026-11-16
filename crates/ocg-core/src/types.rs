//! Type definitions used across the application.

pub mod alliance;
pub mod custom_domain;
pub mod dashboard;
pub mod distribution;
pub mod event;
pub mod group;
pub mod gtm;
pub mod inbox;
pub mod jobs;
pub mod landscape;
pub mod location;
pub mod mock_interviews;
pub mod notifications;
pub mod opportunities;
pub mod pagination;
pub mod partner_integration;
pub mod payments;
pub mod permissions;
pub mod questionnaire;
pub mod search;
pub mod site;
pub mod survey;
pub mod user;
pub mod utm;

#[cfg(any(test, feature = "test-utils"))]
#[path = "types/test_fixtures.rs"]
pub mod tests;
