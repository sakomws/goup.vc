//! Database operations for the GTM pipeline.

use anyhow::Result;
use async_trait::async_trait;
use tokio_postgres::types::Json;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::PgExecutor,
    types::gtm::{
        GtmAgentDraftList, GtmLead, GtmLeadCandidates, GtmLeadFilters, GtmLeadInput, GtmLeadList,
        GtmReviewDraftResult,
    },
};

/// Database operations for GTM leads and agent drafts.
#[async_trait]
pub(crate) trait DBGtm {
    /// List leads for an alliance, optionally narrowed by filters.
    async fn list_gtm_leads(
        &self,
        alliance_id: Uuid,
        filters: &GtmLeadFilters,
    ) -> Result<GtmLeadList>;

    /// Fetch one lead with activity and drafts.
    async fn get_gtm_lead(&self, alliance_id: Uuid, gtm_lead_id: Uuid) -> Result<Option<GtmLead>>;

    /// Create a lead.
    async fn add_gtm_lead(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        input: &GtmLeadInput,
    ) -> Result<Uuid>;

    /// Update lead fields.
    async fn update_gtm_lead(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        gtm_lead_id: Uuid,
        input: &serde_json::Value,
    ) -> Result<()>;

    /// Delete a lead. When `group_id` is set, the lead must belong to that group.
    async fn delete_gtm_lead(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        gtm_lead_id: Uuid,
        group_id: Option<Uuid>,
    ) -> Result<()>;

    /// Move a lead to a new stage.
    async fn transition_gtm_lead(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        gtm_lead_id: Uuid,
        stage: &str,
        human: bool,
        details: &serde_json::Value,
    ) -> Result<()>;

    /// Create an agent draft.
    async fn add_gtm_agent_draft(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        input: &serde_json::Value,
    ) -> Result<Uuid>;

    /// List drafts.
    async fn list_gtm_agent_drafts(
        &self,
        alliance_id: Uuid,
        filters: &serde_json::Value,
    ) -> Result<GtmAgentDraftList>;

    /// Approve or reject a draft, optionally advancing the lead.
    async fn review_gtm_agent_draft(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        draft_id: Uuid,
        input: &serde_json::Value,
    ) -> Result<GtmReviewDraftResult>;

    /// Suggest lead-generation candidates.
    async fn suggest_gtm_lead_candidates(
        &self,
        alliance_id: Uuid,
        group_id: Option<Uuid>,
        limit: i32,
    ) -> Result<GtmLeadCandidates>;
}

#[async_trait]
impl<T> DBGtm for T
where
    T: PgExecutor + Send + Sync,
{
    #[instrument(skip(self, filters), err)]
    async fn list_gtm_leads(
        &self,
        alliance_id: Uuid,
        filters: &GtmLeadFilters,
    ) -> Result<GtmLeadList> {
        self.fetch_json_one(
            "select list_gtm_leads($1::uuid, $2::jsonb)",
            &[&alliance_id, &Json(filters)],
        )
        .await
    }

    #[instrument(skip(self), err)]
    async fn get_gtm_lead(&self, alliance_id: Uuid, gtm_lead_id: Uuid) -> Result<Option<GtmLead>> {
        self.fetch_json_opt(
            "select get_gtm_lead($1::uuid, $2::uuid)",
            &[&alliance_id, &gtm_lead_id],
        )
        .await
    }

    #[instrument(skip(self, input), err)]
    async fn add_gtm_lead(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        input: &GtmLeadInput,
    ) -> Result<Uuid> {
        self.fetch_scalar_one(
            "select add_gtm_lead($1::uuid, $2::uuid, $3::jsonb)",
            &[&actor_user_id, &alliance_id, &Json(input)],
        )
        .await
    }

    #[instrument(skip(self, input), err)]
    async fn update_gtm_lead(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        gtm_lead_id: Uuid,
        input: &serde_json::Value,
    ) -> Result<()> {
        self.execute(
            "select update_gtm_lead($1::uuid, $2::uuid, $3::uuid, $4::jsonb)",
            &[&actor_user_id, &alliance_id, &gtm_lead_id, &Json(input)],
        )
        .await
    }

    #[instrument(skip(self), err)]
    async fn delete_gtm_lead(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        gtm_lead_id: Uuid,
        group_id: Option<Uuid>,
    ) -> Result<()> {
        self.execute(
            "select delete_gtm_lead($1::uuid, $2::uuid, $3::uuid, $4::uuid)",
            &[&actor_user_id, &alliance_id, &gtm_lead_id, &group_id],
        )
        .await
    }

    #[instrument(skip(self, details), err)]
    async fn transition_gtm_lead(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        gtm_lead_id: Uuid,
        stage: &str,
        human: bool,
        details: &serde_json::Value,
    ) -> Result<()> {
        self.execute(
            "select transition_gtm_lead($1::uuid, $2::uuid, $3::uuid, $4::text, $5::boolean, $6::jsonb)",
            &[
                &actor_user_id,
                &alliance_id,
                &gtm_lead_id,
                &stage,
                &human,
                &Json(details),
            ],
        )
        .await
    }

    #[instrument(skip(self, input), err)]
    async fn add_gtm_agent_draft(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        input: &serde_json::Value,
    ) -> Result<Uuid> {
        self.fetch_scalar_one(
            "select add_gtm_agent_draft($1::uuid, $2::uuid, $3::jsonb)",
            &[&actor_user_id, &alliance_id, &Json(input)],
        )
        .await
    }

    #[instrument(skip(self, filters), err)]
    async fn list_gtm_agent_drafts(
        &self,
        alliance_id: Uuid,
        filters: &serde_json::Value,
    ) -> Result<GtmAgentDraftList> {
        self.fetch_json_one(
            "select list_gtm_agent_drafts($1::uuid, $2::jsonb)",
            &[&alliance_id, &Json(filters)],
        )
        .await
    }

    #[instrument(skip(self, input), err)]
    async fn review_gtm_agent_draft(
        &self,
        actor_user_id: Uuid,
        alliance_id: Uuid,
        draft_id: Uuid,
        input: &serde_json::Value,
    ) -> Result<GtmReviewDraftResult> {
        self.fetch_json_one(
            "select review_gtm_agent_draft($1::uuid, $2::uuid, $3::uuid, $4::jsonb)",
            &[&actor_user_id, &alliance_id, &draft_id, &Json(input)],
        )
        .await
    }

    #[instrument(skip(self), err)]
    async fn suggest_gtm_lead_candidates(
        &self,
        alliance_id: Uuid,
        group_id: Option<Uuid>,
        limit: i32,
    ) -> Result<GtmLeadCandidates> {
        self.fetch_json_one(
            "select suggest_gtm_lead_candidates($1::uuid, $2::uuid, $3::int)",
            &[&alliance_id, &group_id, &limit],
        )
        .await
    }
}
