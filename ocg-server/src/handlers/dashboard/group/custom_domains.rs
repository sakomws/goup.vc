//! Dashboard handlers for group and event custom domains.

use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{CurrentUser, SelectedAllianceId, SelectedGroupId, ValidatedFormQs},
    },
    services::custom_domains::verify_txt_record,
    templates::dashboard::group::custom_domains::{Card, CustomDomainInput},
    types::custom_domain::CustomDomain,
};

fn action_url(event_id: Option<Uuid>) -> String {
    event_id.map_or_else(
        || "/dashboard/group/custom-domain".to_string(),
        |event_id| format!("/dashboard/group/events/{event_id}/custom-domain"),
    )
}

fn render_card(
    domain: Option<CustomDomain>,
    event_id: Option<Uuid>,
) -> Result<Html<String>, HandlerError> {
    Ok(Html(
        Card {
            action_url: action_url(event_id),
            can_manage: true,
            domain,
            target_label: if event_id.is_some() { "event" } else { "group" },
        }
        .render()?,
    ))
}

async fn get_card(
    db: &DynDB,
    group_id: Uuid,
    event_id: Option<Uuid>,
) -> Result<Html<String>, HandlerError> {
    render_card(db.get_custom_domain(group_id, event_id).await?, event_id)
}

/// Renders group custom-domain management.
#[instrument(skip_all, err)]
pub(crate) async fn group_card(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    get_card(&db, group_id, None).await
}

/// Renders event custom-domain management.
#[instrument(skip_all, err)]
pub(crate) async fn event_card(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    get_card(&db, group_id, Some(event_id)).await
}

async fn set_domain(
    db: &DynDB,
    actor_user_id: Uuid,
    alliance_id: Uuid,
    group_id: Uuid,
    event_id: Option<Uuid>,
    hostname: &str,
) -> Result<Html<String>, HandlerError> {
    let verification_token = format!("goup-verification={}", Uuid::new_v4());
    let domain = db
        .upsert_custom_domain(
            actor_user_id,
            alliance_id,
            group_id,
            event_id,
            hostname,
            &verification_token,
        )
        .await?;
    render_card(Some(domain), event_id)
}

/// Creates or replaces a group custom domain.
#[instrument(skip_all, err)]
pub(crate) async fn set_group(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    ValidatedFormQs(input): ValidatedFormQs<CustomDomainInput>,
) -> Result<impl IntoResponse, HandlerError> {
    set_domain(
        &db,
        user.user_id,
        alliance_id,
        group_id,
        None,
        &input.hostname,
    )
    .await
}

/// Creates or replaces an event custom domain.
#[instrument(skip_all, err)]
pub(crate) async fn set_event(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
    ValidatedFormQs(input): ValidatedFormQs<CustomDomainInput>,
) -> Result<impl IntoResponse, HandlerError> {
    set_domain(
        &db,
        user.user_id,
        alliance_id,
        group_id,
        Some(event_id),
        &input.hostname,
    )
    .await
}

async fn verify_domain(
    db: &DynDB,
    actor_user_id: Uuid,
    group_id: Uuid,
    event_id: Option<Uuid>,
) -> Result<Html<String>, HandlerError> {
    let domain = db
        .get_custom_domain(group_id, event_id)
        .await?
        .ok_or(HandlerError::NotFound)?;
    if !verify_txt_record(&domain.hostname, &domain.verification_token).await? {
        return Err(HandlerError::Database(format!(
            "TXT record {} does not contain the expected verification token yet.",
            domain.verification_record_name()
        )));
    }
    let domain = db
        .mark_custom_domain_verified(
            actor_user_id,
            group_id,
            event_id,
            domain.custom_domain_id,
            &domain.hostname,
            &domain.verification_token,
        )
        .await?;
    render_card(Some(domain), event_id)
}

/// Verifies DNS ownership for a group custom domain.
#[instrument(skip_all, err)]
pub(crate) async fn verify_group(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    verify_domain(&db, user.user_id, group_id, None).await
}

/// Verifies DNS ownership for an event custom domain.
#[instrument(skip_all, err)]
pub(crate) async fn verify_event(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    verify_domain(&db, user.user_id, group_id, Some(event_id)).await
}

async fn remove_domain(
    db: &DynDB,
    actor_user_id: Uuid,
    group_id: Uuid,
    event_id: Option<Uuid>,
) -> Result<Html<String>, HandlerError> {
    db.delete_custom_domain(actor_user_id, group_id, event_id).await?;
    render_card(None, event_id)
}

/// Removes a group custom domain.
#[instrument(skip_all, err)]
pub(crate) async fn remove_group(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    remove_domain(&db, user.user_id, group_id, None).await
}

/// Removes an event custom domain.
#[instrument(skip_all, err)]
pub(crate) async fn remove_event(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    remove_domain(&db, user.user_id, group_id, Some(event_id)).await
}
