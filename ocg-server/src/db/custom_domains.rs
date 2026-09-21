//! Database operations for custom group and event hostnames.

use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::PgExecutor,
    types::custom_domain::{CustomDomain, CustomDomainTarget},
};

/// Custom-domain persistence and host resolution.
#[async_trait]
pub(crate) trait DBCustomDomains {
    /// Returns the domain assigned to a group or one of its events.
    async fn get_custom_domain(
        &self,
        group_id: Uuid,
        event_id: Option<Uuid>,
    ) -> Result<Option<CustomDomain>>;

    /// Creates or replaces the domain assigned to a group or event.
    #[allow(clippy::too_many_arguments)]
    async fn upsert_custom_domain(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        group_id: Uuid,
        event_id: Option<Uuid>,
        hostname: &str,
        verification_token: &str,
    ) -> Result<CustomDomain>;

    /// Marks a domain as DNS-verified after an ownership check.
    async fn mark_custom_domain_verified(
        &self,
        actor_user_id: Uuid,
        group_id: Uuid,
        event_id: Option<Uuid>,
        custom_domain_id: Uuid,
        hostname: &str,
        verification_token: &str,
    ) -> Result<CustomDomain>;

    /// Removes a custom-domain assignment.
    async fn delete_custom_domain(
        &self,
        actor_user_id: Uuid,
        group_id: Uuid,
        event_id: Option<Uuid>,
    ) -> Result<()>;

    /// Resolves an active hostname to its canonical public path.
    async fn resolve_active_custom_domain(
        &self,
        hostname: &str,
    ) -> Result<Option<CustomDomainTarget>>;
}

#[async_trait]
impl<T> DBCustomDomains for T
where
    T: PgExecutor + Send + Sync,
{
    #[instrument(skip(self), err)]
    async fn get_custom_domain(
        &self,
        group_id: Uuid,
        event_id: Option<Uuid>,
    ) -> Result<Option<CustomDomain>> {
        self.fetch_json_opt(
            "select get_custom_domain($1::uuid, $2::uuid)",
            &[&group_id, &event_id],
        )
        .await
    }

    #[instrument(skip(self, verification_token), err)]
    async fn upsert_custom_domain(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        group_id: Uuid,
        event_id: Option<Uuid>,
        hostname: &str,
        verification_token: &str,
    ) -> Result<CustomDomain> {
        self.fetch_json_one(
            "select upsert_custom_domain($1::uuid, $2::uuid, $3::uuid, $4::uuid, $5::text, $6::text)",
            &[
                &actor_user_id,
                &alliance_id,
                &group_id,
                &event_id,
                &hostname,
                &verification_token,
            ],
        )
        .await
    }

    #[instrument(skip(self), err)]
    async fn mark_custom_domain_verified(
        &self,
        actor_user_id: Uuid,
        group_id: Uuid,
        event_id: Option<Uuid>,
        custom_domain_id: Uuid,
        hostname: &str,
        verification_token: &str,
    ) -> Result<CustomDomain> {
        self.fetch_json_one(
            "select mark_custom_domain_verified($1::uuid, $2::uuid, $3::uuid, $4::uuid, $5::text, $6::text)",
            &[
                &actor_user_id,
                &group_id,
                &event_id,
                &custom_domain_id,
                &hostname,
                &verification_token,
            ],
        )
        .await
    }

    #[instrument(skip(self), err)]
    async fn delete_custom_domain(
        &self,
        actor_user_id: Uuid,
        group_id: Uuid,
        event_id: Option<Uuid>,
    ) -> Result<()> {
        self.execute(
            "select delete_custom_domain($1::uuid, $2::uuid, $3::uuid)",
            &[&actor_user_id, &group_id, &event_id],
        )
        .await
    }

    #[instrument(skip(self), err)]
    async fn resolve_active_custom_domain(
        &self,
        hostname: &str,
    ) -> Result<Option<CustomDomainTarget>> {
        self.fetch_json_opt(
            "select resolve_active_custom_domain($1::text)",
            &[&hostname],
        )
        .await
    }
}
