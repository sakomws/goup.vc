//! Public unified opportunity board handlers.

use askama::Template;
use axum::{
    extract::{Path, RawQuery, State},
    http::{
        HeaderValue, StatusCode,
        header::{CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_TYPE},
    },
    response::{Html, IntoResponse, Response},
};
use garde::Validate;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::error::HandlerError,
    router::{CACHE_CONTROL_PRIVATE_NO_STORE, serde_qs_config},
    templates::{PageId, auth::User, site::opportunities},
    types::{
        opportunities::OpportunityFilters,
        pagination::{NavigationLinks, ToRawQuery},
    },
};

const OPPORTUNITIES_URL: &str = "/opportunities";

#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    let mut filters = parse_filters(raw_query.as_deref().unwrap_or_default())?;
    filters.include_members_only = auth_session.user.is_some();
    let output = db.search_opportunities(&filters).await?;
    let navigation_links = NavigationLinks::from_filters(
        &filters,
        output.total,
        OPPORTUNITIES_URL,
        OPPORTUNITIES_URL,
    )?;
    let csv_url = format!("{OPPORTUNITIES_URL}.csv?{}", filters.to_raw_query()?);
    let template = opportunities::Page {
        page_id: PageId::SiteOpportunities,
        path: OPPORTUNITIES_URL.into(),
        site_settings: db.get_site_settings().await?,
        user: User::from_session(auth_session).await?,
        filters,
        csv_url,
        opportunities: output.opportunities,
        total: output.total,
        navigation_links,
    };
    Ok((
        [(CACHE_CONTROL, CACHE_CONTROL_PRIVATE_NO_STORE)],
        Html(template.render()?),
    ))
}

#[instrument(skip_all, err)]
pub(crate) async fn details(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    Path((source_kind, source_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, HandlerError> {
    let include_members_only = auth_session.user.is_some();
    let opportunity = db
        .get_opportunity(&source_kind, source_id, include_members_only)
        .await?
        .ok_or(HandlerError::NotFound)?;
    let path = format!("{OPPORTUNITIES_URL}/{source_kind}/{source_id}");
    let template = opportunities::DetailsPage {
        page_id: PageId::SiteOpportunities,
        path,
        site_settings: db.get_site_settings().await?,
        user: User::from_session(auth_session).await?,
        opportunity,
    };
    Ok((
        [(CACHE_CONTROL, CACHE_CONTROL_PRIVATE_NO_STORE)],
        Html(template.render()?),
    ))
}

/// Export the visible result set, with the same privacy and publication rules
/// as the HTML board.
#[instrument(skip_all, err)]
pub(crate) async fn csv(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    RawQuery(raw_query): RawQuery,
) -> Result<Response, HandlerError> {
    let mut filters = parse_filters(raw_query.as_deref().unwrap_or_default())?;
    filters.include_members_only = auth_session.user.is_some();
    filters.limit = Some(100);
    filters.offset = Some(0);
    let output = db.search_opportunities(&filters).await?;
    let mut body = String::from("kind,title,organization,summary,location,remote,closes_at,url\n");
    for item in output.opportunities {
        let closes_at = item.closes_at.map(|value| value.to_rfc3339()).unwrap_or_default();
        let row = [
            item.kind,
            item.title,
            item.organization_name,
            item.summary,
            item.location.unwrap_or_default(),
            item.remote.to_string(),
            closes_at,
            item.apply_url,
        ]
        .map(|value| format!("\"{}\"", value.replace('"', "\"\"")));
        body.push_str(&row.join(","));
        body.push('\n');
    }
    Ok((
        StatusCode::OK,
        [
            (
                CONTENT_TYPE,
                HeaderValue::from_static("text/csv; charset=utf-8"),
            ),
            (
                CONTENT_DISPOSITION,
                HeaderValue::from_static("attachment; filename=\"goup-opportunities.csv\""),
            ),
        ],
        body,
    )
        .into_response())
}

fn parse_filters(raw_query: &str) -> Result<OpportunityFilters, HandlerError> {
    let filters = if raw_query.is_empty() {
        OpportunityFilters::default()
    } else {
        serde_qs_config().deserialize_str(raw_query)?
    };
    filters.validate()?;
    Ok(filters)
}
