//! HTTP handlers for listing alliance members in the dashboard.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{RawQuery, State},
    http::{
        HeaderName,
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
        extractors::{CurrentUser, SelectedAllianceId},
    },
    router::serde_qs_config,
    templates::{
        alliance::{AllianceMemberExport, AllianceMembersFilters},
        dashboard::alliance::members,
    },
    types::{
        pagination::{self, NavigationLinks},
        permissions::AlliancePermission,
    },
};

const DASHBOARD_URL: &str = "/dashboard/alliance?tab=members";
const PARTIAL_URL: &str = "/dashboard/alliance/members";

/// Displays members across all groups in the selected alliance.
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

/// Downloads all alliance members, including admin-only contact fields, as CSV.
#[instrument(skip_all, err)]
pub(crate) async fn download_csv(
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    State(db): State<DynDB>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    let mut filters: AllianceMembersFilters =
        serde_qs_config().deserialize_str(raw_query.as_deref().unwrap_or_default())?;
    filters.limit = None;
    filters.offset = None;
    filters.validate()?;

    let (alliance, members) = tokio::try_join!(
        db.get_alliance_full(alliance_id),
        db.list_alliance_members_for_export(alliance_id, &filters)
    )?;
    let csv = build_members_csv(&members)?;
    let file_name = format!("alliance-{}-members.csv", alliance.name);
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

/// Prepares the alliance members list page and filters.
#[instrument(skip(db), err)]
pub(crate) async fn prepare_list_page(
    db: &DynDB,
    alliance_id: Uuid,
    user_id: Uuid,
    raw_query: &str,
) -> Result<(AllianceMembersFilters, members::ListPage)> {
    let filters: AllianceMembersFilters = serde_qs_config().deserialize_str(raw_query)?;
    filters.validate()?;

    let (alliance, output, can_export_members) = tokio::try_join!(
        db.get_alliance_full(alliance_id),
        db.list_alliance_members(alliance_id, &filters),
        db.user_has_alliance_permission(&alliance_id, &user_id, AlliancePermission::GroupsWrite)
    )?;
    let navigation_links =
        NavigationLinks::from_filters(&filters, output.total, DASHBOARD_URL, PARTIAL_URL)?;

    let template = members::ListPage {
        alliance,
        can_export_members,
        members: output.members,
        navigation_links,
        total: output.total,
        limit: filters.limit,
        offset: filters.offset,
        query: filters.query.clone(),
    };

    Ok((filters, template))
}

fn build_members_csv(members: &[AllianceMemberExport]) -> Result<Vec<u8>, HandlerError> {
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::Any(b'\n'))
        .from_writer(vec![]);
    writer
        .write_record([
            "Name", "Username", "Email", "Phone", "Groups", "Company", "Title", "City", "Country",
            "LinkedIn", "GitHub", "Website",
        ])
        .map_err(anyhow::Error::from)?;
    for member in members {
        writer
            .write_record([
                member.name.as_deref().unwrap_or(&member.username),
                &member.username,
                &member.email,
                &format!(
                    "{} {}",
                    member.phone_country_code.as_deref().unwrap_or(""),
                    member.phone_number.as_deref().unwrap_or("")
                ),
                &member.group_names.join(", "),
                member.company.as_deref().unwrap_or(""),
                member.title.as_deref().unwrap_or(""),
                member.city.as_deref().unwrap_or(""),
                member.country.as_deref().unwrap_or(""),
                member.linkedin_url.as_deref().unwrap_or(""),
                member.github_url.as_deref().unwrap_or(""),
                member.website_url.as_deref().unwrap_or(""),
            ])
            .map_err(anyhow::Error::from)?;
    }
    writer.into_inner().map_err(|error| anyhow::Error::from(error).into())
}
