//! Database interfaces for dashboards functionality.
//!
//! This module provides traits and types for dashboards-related database operations.

use async_trait::async_trait;

use crate::PgExecutor;

use alliance::DBDashboardAlliance;
use common::DBDashboardCommon;
use group::DBDashboardGroup;
use user::DBDashboardUser;

/// Alliance dashboard database operations.
pub mod alliance;
/// Common dashboard database operations.
pub mod common;
/// Group dashboard database operations.
pub mod group;
/// User dashboard database operations.
pub mod user;

/// Unified database trait for all dashboards operations.
#[async_trait]
pub trait DBDashboard:
    DBDashboardCommon + DBDashboardAlliance + DBDashboardGroup + DBDashboardUser
{
}

impl<T> DBDashboard for T where T: PgExecutor + Send + Sync {}
