//! HTTP handlers for affiliations in the user dashboard.

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
        extractors::{CurrentUser, ValidatedForm},
    },
    templates::dashboard::user::affiliations::{self, UserAffiliationForm},
};

#[cfg(test)]
mod tests;

/// Returns the affiliations page for the current user.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    let template = prepare_list_page(&db, user.user_id).await?;

    Ok(Html(template.render()?))
}

/// Adds an affiliation for the current user, updating the role if the
/// affiliation already exists.
#[instrument(skip_all, err)]
pub(crate) async fn add(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    ValidatedForm(affiliation): ValidatedForm<UserAffiliationForm>,
) -> Result<impl IntoResponse, HandlerError> {
    db.add_user_affiliation(user.user_id, &affiliation).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-user-dashboard-content")],
    ))
}

/// Deletes one of the current user's affiliations.
#[instrument(skip_all, err)]
pub(crate) async fn delete(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(user_affiliation_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.delete_user_affiliation(user.user_id, user_affiliation_id).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-user-dashboard-content")],
    ))
}

/// Prepares the affiliations list page.
#[instrument(skip(db), err)]
pub(crate) async fn prepare_list_page(
    db: &DynDB,
    user_id: Uuid,
) -> Result<affiliations::ListPage, HandlerError> {
    let (affiliations, entry_options) = tokio::try_join!(
        db.list_user_affiliations(user_id),
        db.list_landscape_entry_options()
    )?;

    Ok(affiliations::ListPage {
        affiliations,
        entry_options,
    })
}
