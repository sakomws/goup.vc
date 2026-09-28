//! Alliance dashboard GTM routes.

use anyhow::Result;
use axum::{
    extract::{Path, RawQuery, State},
    response::IntoResponse,
};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::DynDB,
    handlers::{
        dashboard::gtm::{self as gtm_handlers, GtmScope},
        error::HandlerError,
        extractors::{CurrentUser, SelectedAllianceId, ValidatedForm},
    },
    services::notifications::DynNotificationsManager,
    types::gtm::{
        GtmActivityInput, GtmDeliverableInput, GtmLeadInput, GtmLeadTransitionInput,
        GtmReviewDraftInput, GtmRunAgentInput, GtmSponsorContactInput, GtmSponsorPackageInput,
        GtmSponsorProposalInput, GtmStateInput, GtmTaskInput,
    },
};

#[cfg(test)]
mod tests;

/// Alliance GTM list partial.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    user: CurrentUser,
    db: State<DynDB>,
    raw_query: RawQuery,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::list_page(user, db, raw_query, alliance_id, None, GtmScope::Alliance).await
}

/// Alliance GTM lead detail.
#[instrument(skip_all, err)]
pub(crate) async fn detail_page(
    user: CurrentUser,
    db: State<DynDB>,
    path: Path<Uuid>,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::detail_page(user, db, path, alliance_id, None, GtmScope::Alliance).await
}

/// Creates an alliance-scoped lead.
#[instrument(skip_all, err)]
pub(crate) async fn add(
    user: CurrentUser,
    db: State<DynDB>,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    input: ValidatedForm<GtmLeadInput>,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::add(user, db, alliance_id, None, input).await
}

/// Updates an alliance lead.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    user: CurrentUser,
    db: State<DynDB>,
    path: Path<Uuid>,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    input: ValidatedForm<GtmLeadInput>,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::update(user, db, path, alliance_id, None, input).await
}

/// Deletes an alliance lead.
#[instrument(skip_all, err)]
pub(crate) async fn delete(
    user: CurrentUser,
    db: State<DynDB>,
    path: Path<Uuid>,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::delete(user, db, path, alliance_id, None).await
}

/// Transitions an alliance lead.
#[instrument(skip_all, err)]
pub(crate) async fn transition(
    user: CurrentUser,
    db: State<DynDB>,
    path: Path<Uuid>,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    input: ValidatedForm<GtmLeadTransitionInput>,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::transition(user, db, path, alliance_id, input).await
}

/// Runs an alliance-level agent.
#[instrument(skip_all, err)]
pub(crate) async fn run_agent(
    user: CurrentUser,
    db: State<DynDB>,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    input: ValidatedForm<GtmRunAgentInput>,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::run_agent(user, db, alliance_id, None, None, input).await
}

/// Runs an agent against one alliance lead.
#[instrument(skip_all, err)]
pub(crate) async fn run_lead_agent(
    user: CurrentUser,
    db: State<DynDB>,
    Path(gtm_lead_id): Path<Uuid>,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    input: ValidatedForm<GtmRunAgentInput>,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::run_agent(user, db, alliance_id, None, Some(gtm_lead_id), input).await
}

/// Reviews an alliance draft.
#[instrument(skip_all, err)]
pub(crate) async fn review_draft(
    user: CurrentUser,
    db: State<DynDB>,
    notifications_manager: State<DynNotificationsManager>,
    path: Path<Uuid>,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    input: ValidatedForm<GtmReviewDraftInput>,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::review_draft(user, db, notifications_manager, path, alliance_id, input).await
}

pub(crate) async fn add_package(
    user: CurrentUser,
    db: State<DynDB>,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    input: ValidatedForm<GtmSponsorPackageInput>,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::add_package(user, db, alliance_id, None, input).await
}

macro_rules! alliance_lead_action {
    ($name:ident, $shared:ident, $input:ty) => {
        pub(crate) async fn $name(
            user: CurrentUser,
            db: State<DynDB>,
            path: Path<Uuid>,
            SelectedAllianceId(alliance_id): SelectedAllianceId,
            input: ValidatedForm<$input>,
        ) -> Result<impl IntoResponse, HandlerError> {
            gtm_handlers::$shared(user, db, path, alliance_id, input).await
        }
    };
}

alliance_lead_action!(add_contact, add_contact, GtmSponsorContactInput);
alliance_lead_action!(add_proposal, add_proposal, GtmSponsorProposalInput);
alliance_lead_action!(add_task, add_task, GtmTaskInput);
alliance_lead_action!(add_deliverable, add_deliverable, GtmDeliverableInput);
alliance_lead_action!(add_activity, add_activity, GtmActivityInput);

pub(crate) async fn set_task_state(
    user: CurrentUser,
    db: State<DynDB>,
    path: Path<Uuid>,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    input: ValidatedForm<GtmStateInput>,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::set_campaign_state(user, db, path, alliance_id, None, "task", input).await
}

pub(crate) async fn set_proposal_state(
    user: CurrentUser,
    db: State<DynDB>,
    path: Path<Uuid>,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    input: ValidatedForm<GtmStateInput>,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::set_campaign_state(user, db, path, alliance_id, None, "proposal", input).await
}

pub(crate) async fn set_deliverable_state(
    user: CurrentUser,
    db: State<DynDB>,
    path: Path<Uuid>,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    input: ValidatedForm<GtmStateInput>,
) -> Result<impl IntoResponse, HandlerError> {
    gtm_handlers::set_campaign_state(user, db, path, alliance_id, None, "deliverable", input).await
}

/// Prepares the alliance GTM tab.
pub(crate) async fn prepare_list_page(
    db: &crate::db::DynDB,
    alliance_id: Uuid,
    user_id: Uuid,
    raw_query: &str,
) -> Result<
    (
        crate::types::gtm::GtmLeadFilters,
        crate::templates::dashboard::gtm::ListPage,
    ),
    HandlerError,
> {
    gtm_handlers::prepare_list_page(
        db,
        alliance_id,
        user_id,
        None,
        GtmScope::Alliance,
        raw_query,
    )
    .await
}
