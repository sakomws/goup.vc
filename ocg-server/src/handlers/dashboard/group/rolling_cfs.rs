//! Dashboard handlers for a group's rolling Call for Speakers.

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
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
    templates::dashboard::group::rolling_cfs::{
        self, AssignmentInput, ConfigInput, SubmissionUpdate,
    },
    types::permissions::GroupPermission,
};

/// Displays CFS configuration, the review pool, and assignment history.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    let (can_manage_events, config, events, statuses, submissions) = tokio::try_join!(
        db.user_has_group_permission(
            &alliance_id,
            &group_id,
            &user.user_id,
            GroupPermission::EventsWrite
        ),
        db.get_group_cfs_dashboard(group_id),
        db.list_group_cfs_assignment_events(group_id),
        db.list_cfs_submission_statuses_for_review(),
        db.list_group_cfs_submissions(group_id),
    )?;
    Ok(Html(
        rolling_cfs::Page {
            can_manage_events,
            config,
            events,
            statuses,
            submissions,
        }
        .render()?,
    ))
}

/// Updates the standing CFS configuration.
#[instrument(skip_all, err)]
pub(crate) async fn update_config(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    ValidatedFormQs(input): ValidatedFormQs<ConfigInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.update_group_cfs(group_id, input.enabled, input.description, &input.labels)
        .await?;
    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-rolling-cfs")],
    ))
}

/// Records a status, labels, message, and optional reviewer rating.
#[instrument(skip_all, err)]
pub(crate) async fn update_submission(
    CurrentUser(reviewer): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(submission_id): Path<Uuid>,
    ValidatedFormQs(input): ValidatedFormQs<SubmissionUpdate>,
) -> Result<impl IntoResponse, HandlerError> {
    db.update_group_cfs_submission(reviewer.user_id, group_id, submission_id, &input)
        .await?;
    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-rolling-cfs")],
    ))
}

/// Assigns an approved rolling proposal to a concrete event.
#[instrument(skip_all, err)]
pub(crate) async fn assign_submission(
    CurrentUser(reviewer): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(submission_id): Path<Uuid>,
    ValidatedFormQs(input): ValidatedFormQs<AssignmentInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.assign_group_cfs_submission(reviewer.user_id, group_id, input.event_id, submission_id)
        .await?;
    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-rolling-cfs")],
    ))
}
