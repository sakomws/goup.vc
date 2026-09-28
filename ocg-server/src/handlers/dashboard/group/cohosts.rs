//! HTTP handlers for group event co-host invitations.

use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    config::HttpServerConfig,
    db::{DBExt, DynDB},
    handlers::{
        error::HandlerError,
        extractors::{CurrentUser, SelectedGroupId, ValidatedForm},
    },
    services::notifications::payloads::build_event_cohost_invitation_notification,
    templates::dashboard::group::cohosts::{self, RequestEventCohost},
};

/// Displays pending co-host invitations for the selected group.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    let template = prepare_inbox(&db, user.user_id, group_id).await?;
    Ok(Html(template.render()?))
}

/// Displays co-host controls for an event owned by the selected group.
#[instrument(skip_all, err)]
pub(crate) async fn event_page(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let template = prepare_event_page(&db, user.user_id, group_id, event_id).await?;
    Ok(Html(template.render()?))
}

/// Invites another group to co-host an event.
#[instrument(skip_all, err)]
pub(crate) async fn request(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    State(server_cfg): State<HttpServerConfig>,
    Path(event_id): Path<Uuid>,
    ValidatedForm(input): ValidatedForm<RequestEventCohost>,
) -> Result<impl IntoResponse, HandlerError> {
    db.as_ref()
        .transaction(|tx| {
            Box::pin(async move {
                let event_cohost_id = tx
                    .request_event_cohost(
                        user.user_id,
                        group_id,
                        event_id,
                        input.cohost_group_id,
                        input.message,
                    )
                    .await?;
                let recipients =
                    tx.claim_event_cohost_invitation_recipients(event_cohost_id).await?;
                if recipients.is_empty() {
                    return Ok(());
                }

                let (data, site_settings) = tokio::try_join!(
                    tx.get_event_cohost_notification_data(event_cohost_id),
                    tx.get_site_settings(),
                )?;
                let notification = build_event_cohost_invitation_notification(
                    &data,
                    recipients,
                    &server_cfg,
                    &site_settings,
                )?;
                tx.enqueue_notification(&notification).await
            })
        })
        .await?;

    let template = prepare_event_page(&db, user.user_id, group_id, event_id).await?;
    Ok(Html(template.render()?))
}

/// Accepts a pending co-host invitation for the selected group.
#[instrument(skip_all, err)]
pub(crate) async fn accept(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_cohost_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    decide_and_render(&db, user.user_id, group_id, event_cohost_id, true).await
}

/// Rejects a pending co-host invitation for the selected group.
#[instrument(skip_all, err)]
pub(crate) async fn reject(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_cohost_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    decide_and_render(&db, user.user_id, group_id, event_cohost_id, false).await
}

/// Removes an active co-host relationship.
#[instrument(skip_all, err)]
pub(crate) async fn revoke(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path((event_id, event_cohost_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, HandlerError> {
    db.revoke_event_cohost(user.user_id, group_id, event_cohost_id)
        .await?;

    let template = prepare_event_page(&db, user.user_id, group_id, event_id).await?;
    Ok(Html(template.render()?))
}

async fn decide_and_render(
    db: &DynDB,
    actor_user_id: Uuid,
    group_id: Uuid,
    event_cohost_id: Uuid,
    approve: bool,
) -> Result<Html<String>, HandlerError> {
    db.decide_event_cohost(actor_user_id, group_id, event_cohost_id, approve)
        .await?;
    let template = prepare_inbox(db, actor_user_id, group_id).await?;
    Ok(Html(template.render()?))
}

pub(super) async fn prepare_inbox(
    db: &DynDB,
    actor_user_id: Uuid,
    group_id: Uuid,
) -> Result<cohosts::ListPage, HandlerError> {
    let invitations = db.list_group_event_cohost_inbox(actor_user_id, group_id).await?;
    Ok(cohosts::ListPage { invitations })
}

async fn prepare_event_page(
    db: &DynDB,
    actor_user_id: Uuid,
    group_id: Uuid,
    event_id: Uuid,
) -> Result<cohosts::EventCohostsPage, HandlerError> {
    let (mut candidates, requests) = tokio::try_join!(
        db.list_event_cohost_candidates(actor_user_id, event_id),
        db.list_event_cohost_requests(actor_user_id, group_id, event_id),
    )?;

    candidates.retain(|candidate| {
        !requests.iter().any(|request| {
            request.cohost_group_id == candidate.group_id
                && matches!(request.status.as_str(), "pending" | "approved")
        })
    });

    Ok(cohosts::EventCohostsPage {
        candidates,
        event_id,
        requests,
    })
}
