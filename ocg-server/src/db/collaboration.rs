//! Database operations for group-owned collaboration projects.

use anyhow::Result;
use async_trait::async_trait;
use tokio_postgres::types::Json;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::PgExecutor,
    templates::dashboard::group::projects::{
        CollaborationBookingInput, CollaborationDashboard, CollaborationInvitationInput,
        CollaborationProjectInput, CollaborationUpdateInput, PublicCollaborationProject,
    },
};

/// Collaboration project database contract.
#[async_trait]
pub(crate) trait DBCollaboration {
    async fn get_group_collaboration_dashboard(
        &self,
        group_id: Uuid,
    ) -> Result<CollaborationDashboard>;
    async fn create_collaboration_project(
        &self,
        actor_user_id: Uuid,
        group_id: Uuid,
        input: &CollaborationProjectInput,
    ) -> Result<Uuid>;
    async fn get_public_collaboration_project(
        &self,
        alliance_id: Uuid,
        group_slug: &str,
        project_slug: &str,
    ) -> Result<Option<PublicCollaborationProject>>;
    async fn invite_collaboration_project_member(
        &self,
        actor_user_id: Uuid,
        group_id: Uuid,
        project_id: Uuid,
        input: &CollaborationInvitationInput,
    ) -> Result<Uuid>;
    async fn submit_collaboration_update(
        &self,
        user_id: Uuid,
        project_id: Uuid,
        input: &CollaborationUpdateInput,
    ) -> Result<Uuid>;
    async fn book_collaboration_office_hour(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        input: &CollaborationBookingInput,
    ) -> Result<Uuid>;
}

#[async_trait]
impl<T> DBCollaboration for T
where
    T: PgExecutor + Send + Sync,
{
    #[instrument(skip(self), err)]
    async fn get_group_collaboration_dashboard(
        &self,
        group_id: Uuid,
    ) -> Result<CollaborationDashboard> {
        self.fetch_json_one(
            "select get_group_collaboration_dashboard($1::uuid)",
            &[&group_id],
        )
        .await
    }

    #[instrument(skip(self, input), err)]
    async fn create_collaboration_project(
        &self,
        actor_user_id: Uuid,
        group_id: Uuid,
        input: &CollaborationProjectInput,
    ) -> Result<Uuid> {
        self.fetch_scalar_one(
            "select create_collaboration_project($1::uuid, $2::uuid, $3::jsonb)",
            &[&actor_user_id, &group_id, &Json(input)],
        )
        .await
    }

    #[instrument(skip(self), err)]
    async fn get_public_collaboration_project(
        &self,
        alliance_id: Uuid,
        group_slug: &str,
        project_slug: &str,
    ) -> Result<Option<PublicCollaborationProject>> {
        self.fetch_json_opt(
            "select get_public_collaboration_project($1::uuid, $2::text, $3::text)",
            &[&alliance_id, &group_slug, &project_slug],
        )
        .await
    }

    #[instrument(skip(self, input), err)]
    async fn invite_collaboration_project_member(
        &self,
        actor_user_id: Uuid,
        group_id: Uuid,
        project_id: Uuid,
        input: &CollaborationInvitationInput,
    ) -> Result<Uuid> {
        self.fetch_scalar_one(
            "select invite_collaboration_project_member($1::uuid, $2::uuid, $3::uuid, $4::uuid, $5::text)",
            &[
                &actor_user_id,
                &group_id,
                &project_id,
                &input.user_id,
                &input.role,
            ],
        )
        .await
    }

    #[instrument(skip(self, input), err)]
    async fn submit_collaboration_update(
        &self,
        user_id: Uuid,
        project_id: Uuid,
        input: &CollaborationUpdateInput,
    ) -> Result<Uuid> {
        self.fetch_scalar_one(
            "select submit_collaboration_update($1::uuid, $2::uuid, $3::text, $4::text, $5::text)",
            &[
                &user_id,
                &project_id,
                &input.body,
                &input.blockers,
                &input.next_steps,
            ],
        )
        .await
    }

    #[instrument(skip(self, input), err)]
    async fn book_collaboration_office_hour(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        input: &CollaborationBookingInput,
    ) -> Result<Uuid> {
        self.fetch_scalar_one(
            "select book_collaboration_office_hour($1::uuid, $2::uuid, $3::text)",
            &[&user_id, &session_id, &input.question],
        )
        .await
    }
}
