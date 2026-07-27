//! Group event discovery configuration handlers.

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use axum_messages::Messages;
use garde::Validate;
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{CurrentUser, SelectedAllianceId, SelectedGroupId, ValidatedForm},
    },
    integrations::you_com::{parse_source_urls, validate_source_url},
    services::event_discovery::ManualEventDiscovery,
    templates::dashboard::group::integrations::Page,
    types::permissions::GroupPermission,
};

const MAX_DISCOVERY_SOURCE_IMPORT: usize = 1_000;

/// Displays source URLs, settings, and the latest ingestion result.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    Ok(Html(
        prepare_page(&db, alliance_id, group_id, user.user_id)
            .await?
            .render()?,
    ))
}

/// Starts an authorized discovery run for the selected group.
#[instrument(skip_all, err)]
pub(crate) async fn run(
    SelectedGroupId(group_id): SelectedGroupId,
    State(manual_event_discovery): State<Option<ManualEventDiscovery>>,
) -> Result<impl IntoResponse, HandlerError> {
    let Some(manual_event_discovery) = manual_event_discovery else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    if !manual_event_discovery.enabled() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    manual_event_discovery.spawn_group_run(group_id);
    Ok((
        StatusCode::ACCEPTED,
        [("HX-Trigger", "event-discovery-run-started")],
    )
        .into_response())
}

/// Updates enabled state and location settings.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<SettingsInput>,
) -> Result<impl IntoResponse, HandlerError> {
    validate_settings(&input)?;
    db.update_group_event_integration(
        user.user_id,
        group_id,
        input.enabled,
        input.city.trim(),
        input.timezone.trim(),
    )
    .await?;
    Ok(Html(
        prepare_page(&db, alliance_id, group_id, user.user_id)
            .await?
            .render()?,
    ))
}

/// Adds a source URL after server-side URL validation.
#[instrument(skip_all, err)]
pub(crate) async fn add_source(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<SourceInput>,
) -> Result<impl IntoResponse, HandlerError> {
    validate_source_url(input.url.trim()).map_err(HandlerError::from)?;
    db.add_group_event_integration_source(group_id, input.url.trim())
        .await?;
    Ok(Html(
        prepare_page(&db, alliance_id, group_id, user.user_id)
            .await?
            .render()?,
    ))
}

/// Adds multiple validated source URLs to the selected group.
#[instrument(skip_all, err)]
pub(crate) async fn add_sources(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<SourceBatchInput>,
) -> Result<impl IntoResponse, HandlerError> {
    let parsed = parse_source_urls(&input.urls, MAX_DISCOVERY_SOURCE_IMPORT);
    let submitted_count = parsed.urls.len();
    let urls: Vec<String> = parsed
        .urls
        .into_iter()
        .filter(|url| validate_source_url(url).is_ok())
        .collect();
    let invalid_count = submitted_count - urls.len();
    let added = db.add_group_event_integration_sources(group_id, &urls).await?;
    let duplicate_count = parsed.duplicate_count + urls.len().saturating_sub(added);
    messages.success(format!(
        "Imported {added} source URL(s); skipped {duplicate_count} duplicate(s), \
         {invalid_count} invalid URL(s), and {} above the 1,000 URL limit.",
        parsed.over_limit_count
    ));
    Ok(Html(
        prepare_page(&db, alliance_id, group_id, user.user_id)
            .await?
            .render()?,
    ))
}

/// Deletes one source URL from the selected group only.
#[instrument(skip_all, err)]
pub(crate) async fn delete_source(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(source_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.delete_group_event_integration_source(user.user_id, group_id, source_id)
        .await?;
    Ok(Html(
        prepare_page(&db, alliance_id, group_id, user.user_id)
            .await?
            .render()?,
    ))
}

/// Publishes a reviewed discovered event.
#[instrument(skip_all, err)]
pub(crate) async fn approve_item(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(item_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.approve_group_event_discovery_item(user.user_id, group_id, item_id)
        .await?;
    Ok(Html(
        prepare_page(&db, alliance_id, group_id, user.user_id)
            .await?
            .render()?,
    ))
}

/// Rejects a discovered event and retains its source-scoped audit history.
#[instrument(skip_all, err)]
pub(crate) async fn reject_item(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(item_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.reject_group_event_discovery_item(user.user_id, group_id, item_id)
        .await?;
    Ok(Html(
        prepare_page(&db, alliance_id, group_id, user.user_id)
            .await?
            .render()?,
    ))
}

pub(crate) async fn prepare_page(
    db: &DynDB,
    alliance_id: Uuid,
    group_id: Uuid,
    user_id: Uuid,
) -> Result<Page, HandlerError> {
    let (can_manage_events, mut integration) = tokio::try_join!(
        db.user_has_group_permission(
            &alliance_id,
            &group_id,
            &user_id,
            GroupPermission::EventsWrite
        ),
        db.get_group_event_integration(group_id),
    )?;
    integration.can_manage_events = can_manage_events;
    Ok(Page { integration })
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct SettingsInput {
    #[serde(default)]
    #[garde(skip)]
    enabled: bool,
    #[garde(skip)]
    city: String,
    #[garde(skip)]
    timezone: String,
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct SourceInput {
    #[garde(skip)]
    url: String,
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct SourceBatchInput {
    #[garde(skip)]
    urls: String,
}

fn validate_settings(input: &SettingsInput) -> Result<(), HandlerError> {
    if input.city.trim().is_empty() || input.city.len() > 100 {
        return Err(HandlerError::Database(
            "city must be between 1 and 100 characters".into(),
        ));
    }
    input
        .timezone
        .trim()
        .parse::<chrono_tz::Tz>()
        .map_err(|_| HandlerError::Database("timezone must be a valid IANA timezone".into()))?;
    Ok(())
}
