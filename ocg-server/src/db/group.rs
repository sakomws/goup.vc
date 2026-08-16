//! This module defines database functionality for the group site.

use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::PgExecutor,
    types::{
        event::{EventKind, EventSummary},
        group::{GroupFull, GroupJoinOutcome, GroupMembershipStatus, GroupRollingCfs},
    },
};

/// Database trait defining all data access operations for the group site.
#[async_trait]
pub(crate) trait DBGroup {
    /// Adds a proposal to the group's standing Call for Speakers.
    async fn add_group_cfs_submission(
        &self,
        alliance_id: Uuid,
        group_id: Uuid,
        user_id: Uuid,
        session_proposal_id: Uuid,
        label_ids: &[Uuid],
    ) -> Result<Uuid>;

    /// Retrieves the public rolling CFS configuration for a group.
    async fn get_group_cfs(
        &self,
        alliance_id: Uuid,
        group_slug: &str,
    ) -> Result<Option<GroupRollingCfs>>;

    /// Lists the current user's reusable proposals for a rolling CFS.
    async fn list_user_session_proposals_for_group_cfs(
        &self,
        user_id: Uuid,
        group_id: Uuid,
    ) -> Result<Vec<crate::templates::event::SessionProposal>>;

    /// Retrieves group information.
    async fn get_group_full_by_slug(
        &self,
        alliance_id: Uuid,
        group_slug: &str,
    ) -> Result<Option<GroupFull>>;

    /// Retrieves past events for a specific group.
    async fn get_group_past_events(
        &self,
        alliance_id: Uuid,
        group_slug: &str,
        event_kinds: Vec<EventKind>,
        limit: i32,
    ) -> Result<Vec<EventSummary>>;

    /// Retrieves upcoming events for a specific group.
    async fn get_group_upcoming_events(
        &self,
        alliance_id: Uuid,
        group_slug: &str,
        event_kinds: Vec<EventKind>,
        limit: i32,
    ) -> Result<Vec<EventSummary>>;

    /// Checks if a user is a member of a group.
    async fn is_group_member(
        &self,
        alliance_id: Uuid,
        group_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool>;

    /// Checks the current user's public group membership status.
    async fn get_group_membership_status(
        &self,
        alliance_id: Uuid,
        group_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<GroupMembershipStatus>>;

    /// Adds a user as a member of a group.
    async fn join_group(
        &self,
        alliance_id: Uuid,
        group_id: Uuid,
        user_id: Uuid,
    ) -> Result<GroupJoinOutcome>;

    /// Removes a user from a group.
    async fn leave_group(&self, alliance_id: Uuid, group_id: Uuid, user_id: Uuid) -> Result<()>;
}

#[async_trait]
impl<T> DBGroup for T
where
    T: PgExecutor + Send + Sync,
{
    /// [`DBGroup::add_group_cfs_submission`]
    #[instrument(skip(self), err)]
    async fn add_group_cfs_submission(
        &self,
        alliance_id: Uuid,
        group_id: Uuid,
        user_id: Uuid,
        session_proposal_id: Uuid,
        label_ids: &[Uuid],
    ) -> Result<Uuid> {
        self.fetch_scalar_one(
            "select add_group_cfs_submission($1::uuid, $2::uuid, $3::uuid, $4::uuid, $5::uuid[])::uuid",
            &[&alliance_id, &group_id, &user_id, &session_proposal_id, &label_ids],
        )
        .await
    }

    /// [`DBGroup::get_group_cfs`]
    #[instrument(skip(self), err)]
    async fn get_group_cfs(
        &self,
        alliance_id: Uuid,
        group_slug: &str,
    ) -> Result<Option<GroupRollingCfs>> {
        self.fetch_json_opt(
            "select get_group_cfs($1::uuid, $2::text)",
            &[&alliance_id, &group_slug],
        )
        .await
    }

    /// [`DBGroup::list_user_session_proposals_for_group_cfs`]
    #[instrument(skip(self), err)]
    async fn list_user_session_proposals_for_group_cfs(
        &self,
        user_id: Uuid,
        group_id: Uuid,
    ) -> Result<Vec<crate::templates::event::SessionProposal>> {
        self.fetch_json_one(
            "select list_user_session_proposals_for_group_cfs($1::uuid, $2::uuid)",
            &[&user_id, &group_id],
        )
        .await
    }

    /// [`DBGroup::get_group_full_by_slug`]
    #[instrument(skip(self), err)]
    async fn get_group_full_by_slug(
        &self,
        alliance_id: Uuid,
        group_slug: &str,
    ) -> Result<Option<GroupFull>> {
        self.fetch_json_opt(
            "select get_group_full_by_slug($1::uuid, $2::text)",
            &[&alliance_id, &group_slug],
        )
        .await
    }

    /// [`DB::get_group_past_events`]
    #[instrument(skip(self), err)]
    async fn get_group_past_events(
        &self,
        alliance_id: Uuid,
        group_slug: &str,
        event_kinds: Vec<EventKind>,
        limit: i32,
    ) -> Result<Vec<EventSummary>> {
        let event_kind_ids: Vec<String> = event_kinds.iter().map(ToString::to_string).collect();
        self.fetch_json_one(
            "select get_group_past_events($1::uuid, $2::text, $3::text[], $4::int)",
            &[&alliance_id, &group_slug, &event_kind_ids, &limit],
        )
        .await
    }

    /// [`DB::get_group_upcoming_events`]
    #[instrument(skip(self), err)]
    async fn get_group_upcoming_events(
        &self,
        alliance_id: Uuid,
        group_slug: &str,
        event_kinds: Vec<EventKind>,
        limit: i32,
    ) -> Result<Vec<EventSummary>> {
        let event_kind_ids: Vec<String> = event_kinds.iter().map(ToString::to_string).collect();
        self.fetch_json_one(
            "select get_group_upcoming_events($1::uuid, $2::text, $3::text[], $4::int)",
            &[&alliance_id, &group_slug, &event_kind_ids, &limit],
        )
        .await
    }

    /// [`DB::is_group_member`]
    #[instrument(skip(self), err)]
    async fn is_group_member(
        &self,
        alliance_id: Uuid,
        group_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool> {
        self.fetch_scalar_one(
            "select is_group_member($1::uuid, $2::uuid, $3::uuid)",
            &[&alliance_id, &group_id, &user_id],
        )
        .await
    }

    /// [`DB::get_group_membership_status`]
    #[instrument(skip(self), err)]
    async fn get_group_membership_status(
        &self,
        alliance_id: Uuid,
        group_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<GroupMembershipStatus>> {
        self.fetch_json_opt(
            "select get_group_membership_status($1::uuid, $2::uuid, $3::uuid)",
            &[&alliance_id, &group_id, &user_id],
        )
        .await
    }

    /// [`DB::join_group`]
    #[instrument(skip(self), err)]
    async fn join_group(
        &self,
        alliance_id: Uuid,
        group_id: Uuid,
        user_id: Uuid,
    ) -> Result<GroupJoinOutcome> {
        let status: String = self
            .fetch_scalar_one(
                "select join_group($1::uuid, $2::uuid, $3::uuid)::text",
                &[&alliance_id, &group_id, &user_id],
            )
            .await?;

        status.parse().map_err(|_| {
            anyhow::anyhow!("unknown group join outcome returned by database: {status}")
        })
    }

    /// [`DB::leave_group`]
    #[instrument(skip(self), err)]
    async fn leave_group(&self, alliance_id: Uuid, group_id: Uuid, user_id: Uuid) -> Result<()> {
        self.execute(
            "select leave_group($1::uuid, $2::uuid, $3::uuid)",
            &[&alliance_id, &group_id, &user_id],
        )
        .await
    }
}
