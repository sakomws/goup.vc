//! HTTP handlers for managing alliance landscape entries.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Path, RawQuery, State},
    http::{
        HeaderName, StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    },
    response::{Html, IntoResponse},
};
use garde::Validate;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{CurrentUser, SelectedAllianceId, ValidatedFormQs},
    },
    router::serde_qs_config,
    templates::dashboard::alliance::landscape,
    types::{
        landscape::{DashboardLandscapeFilters, LandscapeEntry, LandscapeEntryInput},
        pagination::{self, NavigationLinks},
        permissions::AlliancePermission,
    },
};

const DASHBOARD_URL: &str = "/dashboard/alliance?tab=landscape";
const PARTIAL_URL: &str = "/dashboard/alliance/landscape";

/// Displays the list of landscape entries for the alliance dashboard.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    State(db): State<DynDB>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    let (filters, template) = prepare_list_page(
        &db,
        alliance_id,
        user.user_id,
        raw_query.as_deref().unwrap_or_default(),
    )
    .await?;

    let url = pagination::build_url(DASHBOARD_URL, &filters)?;
    let headers = [(HeaderName::from_static("hx-push-url"), url)];

    Ok((headers, Html(template.render()?)))
}

/// Downloads all matching landscape entries as an administrator-only CSV.
#[instrument(skip_all, err)]
pub(crate) async fn download_csv(
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    State(db): State<DynDB>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    let mut filters: DashboardLandscapeFilters =
        if raw_query.as_deref().unwrap_or_default().is_empty() {
            DashboardLandscapeFilters::default()
        } else {
            serde_qs_config().deserialize_str(raw_query.as_deref().unwrap_or_default())?
        };
    filters.limit = None;
    filters.offset = None;
    filters.validate()?;

    let (alliance, output) = tokio::try_join!(
        db.get_alliance_full(alliance_id),
        db.list_alliance_landscape_entries_for_export(alliance_id, &filters)
    )?;
    let csv = build_landscape_csv(&output.entries)?;
    let file_name = format!("alliance-{}-landscape.csv", alliance.name);
    Ok((
        [
            (CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                CONTENT_DISPOSITION,
                format!("attachment; filename=\"{file_name}\""),
            ),
        ],
        csv,
    ))
}

/// Adds a new landscape entry.
#[instrument(skip_all, err)]
pub(crate) async fn add(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    State(db): State<DynDB>,
    ValidatedFormQs(input): ValidatedFormQs<LandscapeEntryInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.add_landscape_entry(user.user_id, alliance_id, &input).await?;

    Ok((
        StatusCode::CREATED,
        [("HX-Trigger", "refresh-alliance-dashboard-table")],
    ))
}

/// Updates an existing landscape entry.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    State(db): State<DynDB>,
    Path(entry_id): Path<Uuid>,
    ValidatedFormQs(input): ValidatedFormQs<LandscapeEntryInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.update_landscape_entry(user.user_id, alliance_id, entry_id, &input)
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-alliance-dashboard-table")],
    ))
}

/// Deletes a landscape entry.
#[instrument(skip_all, err)]
pub(crate) async fn delete(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    State(db): State<DynDB>,
    Path(entry_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.delete_landscape_entry(user.user_id, alliance_id, entry_id).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-alliance-dashboard-table")],
    ))
}

/// Publishes a landscape entry.
#[instrument(skip_all, err)]
pub(crate) async fn publish(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    State(db): State<DynDB>,
    Path(entry_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.update_landscape_entry_published(user.user_id, alliance_id, entry_id, true)
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-alliance-dashboard-table")],
    ))
}

/// Unpublishes a landscape entry.
#[instrument(skip_all, err)]
pub(crate) async fn unpublish(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    State(db): State<DynDB>,
    Path(entry_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.update_landscape_entry_published(user.user_id, alliance_id, entry_id, false)
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-alliance-dashboard-table")],
    ))
}

/// Prepares the landscape list page and filters for the alliance dashboard.
pub(crate) async fn prepare_list_page(
    db: &DynDB,
    alliance_id: Uuid,
    user_id: Uuid,
    raw_query: &str,
) -> Result<(DashboardLandscapeFilters, landscape::ListPage), HandlerError> {
    let filters: DashboardLandscapeFilters = if raw_query.is_empty() {
        DashboardLandscapeFilters::default()
    } else {
        serde_qs_config().deserialize_str(raw_query)?
    };
    filters.validate()?;

    let (can_manage_landscape, output) = tokio::try_join!(
        db.user_has_alliance_permission(&alliance_id, &user_id, AlliancePermission::GroupsWrite),
        db.list_alliance_landscape_entries(alliance_id, &filters)
    )?;
    let navigation_links =
        NavigationLinks::from_filters(&filters, output.total, DASHBOARD_URL, PARTIAL_URL)?;

    Ok((
        filters.clone(),
        landscape::ListPage {
            can_manage_landscape,
            filters,
            entries: output.entries,
            total: output.total,
            navigation_links,
        },
    ))
}

fn build_landscape_csv(entries: &[LandscapeEntry]) -> Result<Vec<u8>, HandlerError> {
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::Any(b'\n'))
        .from_writer(vec![]);
    writer
        .write_record([
            "Name",
            "Slug",
            "Kind",
            "Published",
            "Category",
            "Summary",
            "Description",
            "Website URL",
            "GitHub URL",
            "Tags",
            "Affiliations",
            "Created At",
            "Updated At",
        ])
        .map_err(anyhow::Error::from)?;
    for entry in entries {
        let row = vec![
            entry.name.clone(),
            entry.slug.clone(),
            entry.kind.clone(),
            if entry.published {
                "Yes".to_string()
            } else {
                "No".to_string()
            },
            entry.category.clone().unwrap_or_default(),
            entry.summary.clone(),
            entry.description.clone().unwrap_or_default(),
            entry.website_url.clone().unwrap_or_default(),
            entry.github_url.clone().unwrap_or_default(),
            entry.tags.join(", "),
            entry
                .affiliations
                .iter()
                .map(|affiliation| {
                    format!(
                        "{} ({})",
                        affiliation.display_name(),
                        affiliation.role_label()
                    )
                })
                .collect::<Vec<_>>()
                .join("; "),
            entry.created_at.format("%Y-%m-%d").to_string(),
            entry
                .updated_at
                .map(|updated_at| updated_at.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
        ];
        writer.write_record(row).map_err(anyhow::Error::from)?;
    }
    writer.into_inner().map_err(|error| anyhow::Error::from(error).into())
}
