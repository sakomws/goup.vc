//! Organizer workflows for collaboration projects.

use askama::Template;
use axum::{
    extract::{Path, State},
    http::{HeaderName, StatusCode},
    response::{Html, IntoResponse},
};
use tracing::instrument;

use crate::{
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{CurrentUser, SelectedAllianceId, SelectedGroupId, ValidatedForm},
    },
    templates::dashboard::group::projects::{
        self, CollaborationInvitationInput, CollaborationProjectInput,
    },
    types::permissions::GroupPermission,
};

const DASHBOARD_URL: &str = "/dashboard/group?tab=projects";

#[instrument(skip_all, err)]
pub(crate) async fn page(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    let template = prepare_page(&db, alliance_id, group_id, user.user_id).await?;
    Ok((
        [(
            HeaderName::from_static("hx-push-url"),
            DASHBOARD_URL.to_string(),
        )],
        Html(template.render()?),
    ))
}

#[instrument(skip_all, err)]
pub(crate) async fn create(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<CollaborationProjectInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.create_collaboration_project(user.user_id, group_id, &input)
        .await?;
    Ok((
        StatusCode::CREATED,
        [("HX-Trigger", "refresh-group-dashboard-table")],
    ))
}

#[instrument(skip_all, err)]
pub(crate) async fn invite_member(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(project_id): Path<uuid::Uuid>,
    ValidatedForm(input): ValidatedForm<CollaborationInvitationInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.invite_collaboration_project_member(user.user_id, group_id, project_id, &input)
        .await?;
    Ok((
        StatusCode::CREATED,
        [("HX-Trigger", "refresh-group-dashboard-table")],
    ))
}

pub(crate) async fn prepare_page(
    db: &DynDB,
    alliance_id: uuid::Uuid,
    group_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<projects::Page, HandlerError> {
    let (can_manage_projects, dashboard) = tokio::try_join!(
        db.user_has_group_permission(
            &alliance_id,
            &group_id,
            &user_id,
            GroupPermission::ProjectsWrite,
        ),
        db.get_group_collaboration_dashboard(group_id),
    )?;
    Ok(projects::Page {
        can_manage_projects,
        dashboard,
    })
}
