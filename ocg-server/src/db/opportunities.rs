//! Database operations for the unified opportunity board.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tokio_postgres::types::Json;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::PgExecutor,
    types::opportunities::{
        DashboardOpportunityFilters, OpportunitiesOutput, OpportunityFilters, OpportunityInput,
        OpportunitySavedSearch, OpportunitySavedSearchInput, OpportunitySummary, parse_tags,
    },
};

#[async_trait]
pub(crate) trait DBOpportunities {
    async fn search_opportunities(
        &self,
        filters: &OpportunityFilters,
    ) -> Result<OpportunitiesOutput>;
    async fn get_opportunity(
        &self,
        source_kind: &str,
        source_id: Uuid,
        include_members_only: bool,
    ) -> Result<Option<OpportunitySummary>>;
    async fn list_user_opportunities(
        &self,
        user_id: Uuid,
        filters: &DashboardOpportunityFilters,
    ) -> Result<OpportunitiesOutput>;
    async fn add_opportunity(&self, user_id: Uuid, input: &OpportunityInput) -> Result<Uuid>;
    async fn update_opportunity(
        &self,
        user_id: Uuid,
        opportunity_id: Uuid,
        input: &OpportunityInput,
    ) -> Result<()>;
    async fn delete_opportunity(&self, user_id: Uuid, opportunity_id: Uuid) -> Result<()>;
    async fn update_opportunity_published(
        &self,
        user_id: Uuid,
        opportunity_id: Uuid,
        published: bool,
    ) -> Result<()>;
    async fn list_opportunity_saved_searches(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<OpportunitySavedSearch>>;
    async fn upsert_opportunity_saved_search(
        &self,
        user_id: Uuid,
        saved_search_id: Option<Uuid>,
        input: &OpportunitySavedSearchInput,
    ) -> Result<Uuid>;
    async fn activate_opportunity_saved_search(
        &self,
        user_id: Uuid,
        saved_search_id: Uuid,
    ) -> Result<()>;
    async fn delete_opportunity_saved_search(
        &self,
        user_id: Uuid,
        saved_search_id: Uuid,
    ) -> Result<()>;
}

#[async_trait]
impl<T> DBOpportunities for T
where
    T: PgExecutor + Send + Sync,
{
    #[instrument(skip(self, filters), err)]
    async fn search_opportunities(
        &self,
        filters: &OpportunityFilters,
    ) -> Result<OpportunitiesOutput> {
        self.fetch_json_one("select search_opportunities($1::jsonb)", &[&Json(filters)])
            .await
    }

    #[instrument(skip(self), err)]
    async fn get_opportunity(
        &self,
        source_kind: &str,
        source_id: Uuid,
        include_members_only: bool,
    ) -> Result<Option<OpportunitySummary>> {
        self.fetch_json_one(
            "select get_opportunity($1::text, $2::uuid, $3::boolean)",
            &[&source_kind, &source_id, &include_members_only],
        )
        .await
    }

    #[instrument(skip(self, filters), err)]
    async fn list_user_opportunities(
        &self,
        user_id: Uuid,
        filters: &DashboardOpportunityFilters,
    ) -> Result<OpportunitiesOutput> {
        self.fetch_json_one(
            "select list_user_opportunities($1::uuid, $2::jsonb)",
            &[&user_id, &Json(filters)],
        )
        .await
    }

    #[instrument(skip(self, input), err)]
    async fn add_opportunity(&self, user_id: Uuid, input: &OpportunityInput) -> Result<Uuid> {
        let tags = parse_tags(input.tags.as_deref());
        self.fetch_scalar_one(
            "select add_opportunity($1::uuid, $2::jsonb, $3::text[])",
            &[&user_id, &Json(input), &tags],
        )
        .await
    }

    #[instrument(skip(self, input), err)]
    async fn update_opportunity(
        &self,
        user_id: Uuid,
        opportunity_id: Uuid,
        input: &OpportunityInput,
    ) -> Result<()> {
        let tags = parse_tags(input.tags.as_deref());
        self.execute(
            "select update_opportunity($1::uuid, $2::uuid, $3::jsonb, $4::text[])",
            &[&user_id, &opportunity_id, &Json(input), &tags],
        )
        .await?;
        Ok(())
    }

    async fn delete_opportunity(&self, user_id: Uuid, opportunity_id: Uuid) -> Result<()> {
        self.execute(
            "select delete_opportunity($1::uuid, $2::uuid)",
            &[&user_id, &opportunity_id],
        )
        .await?;
        Ok(())
    }

    async fn update_opportunity_published(
        &self,
        user_id: Uuid,
        opportunity_id: Uuid,
        published: bool,
    ) -> Result<()> {
        self.execute(
            "select update_opportunity_published($1::uuid, $2::uuid, $3::boolean)",
            &[&user_id, &opportunity_id, &published],
        )
        .await?;
        Ok(())
    }

    async fn list_opportunity_saved_searches(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<OpportunitySavedSearch>> {
        self.fetch_json_one(
            "select list_opportunity_saved_searches($1::uuid)",
            &[&user_id],
        )
        .await
    }

    async fn upsert_opportunity_saved_search(
        &self,
        user_id: Uuid,
        saved_search_id: Option<Uuid>,
        input: &OpportunitySavedSearchInput,
    ) -> Result<Uuid> {
        let payload = json!({
            "name": input.name,
            "frequency": input.frequency,
            "filters": input.filters()
        });
        self.fetch_scalar_one(
            "select upsert_opportunity_saved_search($1::uuid, $2::uuid, $3::jsonb)",
            &[&user_id, &saved_search_id, &Json(payload)],
        )
        .await
    }

    async fn activate_opportunity_saved_search(
        &self,
        user_id: Uuid,
        saved_search_id: Uuid,
    ) -> Result<()> {
        self.execute(
            "select activate_opportunity_saved_search($1::uuid, $2::uuid)",
            &[&user_id, &saved_search_id],
        )
        .await?;
        Ok(())
    }

    async fn delete_opportunity_saved_search(
        &self,
        user_id: Uuid,
        saved_search_id: Uuid,
    ) -> Result<()> {
        self.execute(
            "select delete_opportunity_saved_search($1::uuid, $2::uuid)",
            &[&user_id, &saved_search_id],
        )
        .await?;
        Ok(())
    }
}
