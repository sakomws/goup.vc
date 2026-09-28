//! Organizer CRUD, moderation and saved-search handlers.

use askama::Template;
use axum::{
    extract::{Path, Query, RawQuery, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use axum_messages::Messages;
use garde::Validate;
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{CurrentUser, ValidatedForm},
    },
    router::serde_qs_config,
    templates::{PageId, auth::User, dashboard::opportunities::Page},
    types::{
        opportunities::{
            DashboardOpportunityFilters, OpportunityFilters, OpportunityInput,
            OpportunitySavedSearchInput,
        },
        pagination::NavigationLinks,
    },
};

const DASHBOARD_URL: &str = "/dashboard/opportunities";

#[derive(Debug, Default, Deserialize)]
pub(crate) struct PreviewQuery {
    preview: Option<Uuid>,
}

#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Query(preview_query): Query<PreviewQuery>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    let filters = parse_filters(raw_query.as_deref().unwrap_or_default())?;
    let output = db.list_user_opportunities(user.user_id, &filters).await?;
    let navigation_links =
        NavigationLinks::from_filters(&filters, output.total, DASHBOARD_URL, DASHBOARD_URL)?;
    let saved_searches = db.list_opportunity_saved_searches(user.user_id).await?;
    let preview_search = preview_query.preview.and_then(|id| {
        saved_searches
            .iter()
            .find(|search| search.opportunity_saved_search_id == id)
    });
    let (preview_name, preview) = if let Some(search) = preview_search {
        let mut search_filters: OpportunityFilters =
            serde_json::from_value(search.filters.clone())?;
        search_filters.include_members_only = true;
        (
            Some(search.name.clone()),
            Some(db.search_opportunities(&search_filters).await?),
        )
    } else {
        (None, None)
    };
    let template = Page {
        messages: messages.into_iter().collect(),
        page_id: PageId::OpportunitiesDashboard,
        path: DASHBOARD_URL.into(),
        site_settings: db.get_site_settings().await?,
        user: User::from_session(auth_session).await?,
        filters,
        opportunities: output.opportunities,
        total: output.total,
        navigation_links,
        saved_searches,
        preview_name,
        preview,
    };
    Ok(Html(template.render()?))
}

#[instrument(skip_all, err)]
pub(crate) async fn add(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<OpportunityInput>,
) -> Result<impl IntoResponse, HandlerError> {
    if input.valid_kind() {
        db.add_opportunity(user.user_id, &input).await?;
        messages.success("Opportunity created as a draft.");
    } else {
        messages.error("Choose a supported native opportunity kind.");
    }
    Ok((StatusCode::SEE_OTHER, [("Location", DASHBOARD_URL)]))
}

#[instrument(skip_all, err)]
pub(crate) async fn update(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(opportunity_id): Path<Uuid>,
    ValidatedForm(input): ValidatedForm<OpportunityInput>,
) -> Result<impl IntoResponse, HandlerError> {
    if input.valid_kind() {
        db.update_opportunity(user.user_id, opportunity_id, &input).await?;
        messages.success("Opportunity updated.");
    } else {
        messages.error("Choose a supported native opportunity kind.");
    }
    Ok((StatusCode::SEE_OTHER, [("Location", DASHBOARD_URL)]))
}

pub(crate) async fn delete(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(opportunity_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.delete_opportunity(user.user_id, opportunity_id).await?;
    messages.success("Opportunity deleted.");
    Ok((StatusCode::SEE_OTHER, [("Location", DASHBOARD_URL)]))
}

pub(crate) async fn publish(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(opportunity_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.update_opportunity_published(user.user_id, opportunity_id, true)
        .await?;
    messages.success("Opportunity published.");
    Ok((StatusCode::SEE_OTHER, [("Location", DASHBOARD_URL)]))
}

pub(crate) async fn unpublish(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(opportunity_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.update_opportunity_published(user.user_id, opportunity_id, false)
        .await?;
    messages.success("Opportunity unpublished.");
    Ok((StatusCode::SEE_OTHER, [("Location", DASHBOARD_URL)]))
}

#[instrument(skip_all, err)]
pub(crate) async fn save_search(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<OpportunitySavedSearchInput>,
) -> Result<impl IntoResponse, HandlerError> {
    if !input.valid_frequency() {
        messages.error("Digest frequency must be daily or weekly.");
        return Ok((
            StatusCode::SEE_OTHER,
            [("Location", DASHBOARD_URL.to_string())],
        ));
    }
    let id = db.upsert_opportunity_saved_search(user.user_id, None, &input).await?;
    messages.success("Search saved. Review the preview, then activate it.");
    Ok((
        StatusCode::SEE_OTHER,
        [("Location", format!("{DASHBOARD_URL}?preview={id}"))],
    ))
}

pub(crate) async fn activate_search(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(saved_search_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.activate_opportunity_saved_search(user.user_id, saved_search_id)
        .await?;
    messages.success("Saved search activated.");
    Ok((StatusCode::SEE_OTHER, [("Location", DASHBOARD_URL)]))
}

pub(crate) async fn delete_search(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(saved_search_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.delete_opportunity_saved_search(user.user_id, saved_search_id)
        .await?;
    messages.success("Saved search deleted.");
    Ok((StatusCode::SEE_OTHER, [("Location", DASHBOARD_URL)]))
}

fn parse_filters(raw_query: &str) -> Result<DashboardOpportunityFilters, HandlerError> {
    let query = raw_query
        .split('&')
        .filter(|part| !part.starts_with("preview="))
        .collect::<Vec<_>>()
        .join("&");
    let filters = if query.is_empty() {
        DashboardOpportunityFilters::default()
    } else {
        serde_qs_config().deserialize_str(&query)?
    };
    filters.validate()?;
    Ok(filters)
}
