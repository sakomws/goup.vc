//! Core domain types, configuration, validation, and integrations for OCG.

pub mod auth;
pub mod config;
pub mod integrations;
pub mod shared;
#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;
pub mod types;
pub mod util;
pub mod validation;
