//! HTTP handlers for the Contact tab of the alliance dashboard.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    config::HttpServerConfig,
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{CurrentUser, SelectedAllianceId, ValidatedFormQs},
    },
    services::notifications::enqueue::enqueue_tracked_alliance_custom_notification,
    templates::dashboard::alliance::contact,
    types::{
        dashboard::alliance::contact::AllianceContactFilters,
        notifications::{AllianceCustomNotificationInput, CustomNotificationContent},
        permissions::AlliancePermission,
    },
};

#[cfg(test)]
mod tests;

// Pages handlers.

/// Displays the contact page used to email group team members.
#[instrument(skip_all)]
pub(crate) async fn page(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    // Prepare template
    let alliance = db.get_alliance_summary(alliance_id).await?;
    let template = prepare_page(&db, alliance_id, user.user_id, alliance.display_name).await?;

    Ok(Html(template.render()?))
}

/// Displays the summary of the group team members matching the filters.
///
/// The filters arrive in a form body, as a long group selection would not fit
/// in a request line.
#[instrument(skip_all)]
pub(crate) async fn recipients(
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    State(db): State<DynDB>,
    ValidatedFormQs(form): ValidatedFormQs<AllianceContactRecipientsForm>,
) -> Result<impl IntoResponse, HandlerError> {
    // Summarize the recipients matching the filters
    let summary = db
        .get_alliance_contact_recipients_summary(alliance_id, &form.filters)
        .await?;

    // Prepare template
    let template = contact::RecipientsSummary {
        filters_key: form.filters.to_canonical_query(),
        summary,
    };

    Ok(Html(template.render()?))
}

// Actions handlers.

/// Sends a custom notification to the group team members matching the filters.
#[instrument(skip_all)]
pub(crate) async fn send_alliance_custom_notification(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    State(db): State<DynDB>,
    State(server_cfg): State<HttpServerConfig>,
    ValidatedFormQs(notification): ValidatedFormQs<AllianceCustomNotification>,
) -> Result<impl IntoResponse, HandlerError> {
    // Resolve recipients, enqueue and audit the notification atomically
    enqueue_tracked_alliance_custom_notification(
        db.as_ref(),
        &server_cfg,
        &AllianceCustomNotificationInput {
            actor_user_id: user.user_id,
            alliance_id,
            content: notification.content,
            filters: notification.filters,
        },
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

// Types.

/// Form data for the recipients summary.
///
/// Fields other than the filters, such as the message of the enclosing contact
/// form, are ignored.
#[derive(Debug, Default, Deserialize, Serialize, Validate)]
pub(crate) struct AllianceContactRecipientsForm {
    /// Filters selecting the group team members to summarize.
    #[serde(default)]
    #[garde(dive)]
    pub filters: AllianceContactFilters,
}

/// Form data for alliance custom notifications.
#[derive(Debug, Deserialize, Serialize, Validate)]
pub(crate) struct AllianceCustomNotification {
    /// Subject and body of the notification.
    #[serde(flatten)]
    #[garde(dive)]
    pub content: CustomNotificationContent,

    /// Filters selecting the group team members to notify.
    #[serde(default)]
    #[garde(dive)]
    pub filters: AllianceContactFilters,
}

// Helpers.

/// Prepares the Contact tab for the alliance dashboard.
pub(crate) async fn prepare_page(
    db: &DynDB,
    alliance_id: Uuid,
    user_id: Uuid,
    default_subject: String,
) -> Result<contact::Page, HandlerError> {
    // Load permissions, filter options and the summary of the default filters
    let filters = AllianceContactFilters::default();
    let (can_send, filter_options, roles, summary) = tokio::try_join!(
        db.user_has_alliance_permission(&alliance_id, &user_id, AlliancePermission::GroupsWrite),
        db.list_alliance_contact_filter_options(alliance_id),
        db.list_group_roles(),
        db.get_alliance_contact_recipients_summary(alliance_id, &filters),
    )?;

    // Prepare template
    let template = contact::Page {
        can_send,
        default_notification_subject: default_subject,
        filter_options,
        filters_key: filters.to_canonical_query(),
        roles,
        summary,
    };

    Ok(template)
}
