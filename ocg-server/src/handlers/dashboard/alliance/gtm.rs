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
    types::gtm::{GtmLeadInput, GtmLeadTransitionInput, GtmReviewDraftInput, GtmRunAgentInput},
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
