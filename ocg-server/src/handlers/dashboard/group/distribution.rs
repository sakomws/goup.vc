//! Planning-only group distribution dashboard handlers.

use askama::Template;
use axum::{
    extract::{Path, State},
    http::{HeaderName, StatusCode, header},
    response::{Html, IntoResponse},
};
use uuid::Uuid;

use crate::{
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{CurrentUser, SelectedAllianceId, SelectedGroupId, ValidatedForm},
    },
    templates::dashboard::group::distribution::Page,
    types::{
        distribution::{
            CampaignInput, ContentInput, ContentStateInput, LibraryInput, LinkInput, PartnerInput,
        },
        permissions::GroupPermission,
    },
};

const REFRESH: [(&str, &str); 1] = [("HX-Trigger", "refresh-group-dashboard-table")];

fn require_choice(value: &str, allowed: &[&str], field: &str) -> Result<(), HandlerError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(HandlerError::Deserialization(format!("invalid {field}")))
    }
}

fn csv_safe(value: &str) -> String {
    if value.trim_start().starts_with(['=', '+', '-', '@']) {
        format!("'{value}")
    } else {
        value.to_string()
    }
}

pub(crate) async fn page(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    let page = prepare_page(&db, alliance_id, group_id, user.user_id).await?;
    Ok((
        [(
            HeaderName::from_static("hx-push-url"),
            "/dashboard/group?tab=distribution",
        )],
        Html(page.render()?),
    ))
}

pub(crate) async fn prepare_page(
    db: &DynDB,
    alliance_id: Uuid,
    group_id: Uuid,
    user_id: Uuid,
) -> Result<Page, HandlerError> {
    let (can_manage, dashboard) = tokio::try_join!(
        db.user_has_group_permission(
            &alliance_id,
            &group_id,
            &user_id,
            GroupPermission::DistributionWrite,
        ),
        db.get_distribution_dashboard(group_id),
    )?;
    Ok(Page {
        can_manage,
        dashboard,
    })
}

pub(crate) async fn add_campaign(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<CampaignInput>,
) -> Result<impl IntoResponse, HandlerError> {
    require_choice(
        &input.status,
        &["draft", "active", "completed"],
        "campaign status",
    )?;
    db.add_distribution_campaign(user.user_id, group_id, &input).await?;
    Ok((StatusCode::CREATED, REFRESH))
}

pub(crate) async fn add_partner(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<PartnerInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.add_distribution_partner(group_id, &input).await?;
    Ok((StatusCode::CREATED, REFRESH))
}

pub(crate) async fn add_link(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<LinkInput>,
) -> Result<impl IntoResponse, HandlerError> {
    require_choice(
        &input.channel,
        &["linkedin", "x", "instagram", "email", "partner", "other"],
        "link channel",
    )?;
    super::super::super::distribution::validate_redirect_target(&input.target_url)?;
    db.add_distribution_link(group_id, &input).await?;
    Ok((StatusCode::CREATED, REFRESH))
}

pub(crate) async fn add_content(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<ContentInput>,
) -> Result<impl IntoResponse, HandlerError> {
    require_choice(
        &input.channel,
        &["linkedin", "x", "instagram"],
        "content channel",
    )?;
    require_choice(&input.state, &["idea", "draft", "ready"], "content state")?;
    db.add_distribution_content(group_id, &input).await?;
    Ok((StatusCode::CREATED, REFRESH))
}

pub(crate) async fn add_library_item(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<LibraryInput>,
) -> Result<impl IntoResponse, HandlerError> {
    require_choice(
        &input.kind,
        &["caption", "cta", "hashtags", "event_image"],
        "library item kind",
    )?;
    db.add_distribution_library_item(group_id, &input).await?;
    Ok((StatusCode::CREATED, REFRESH))
}

pub(crate) async fn update_content_state(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(content_id): Path<Uuid>,
    ValidatedForm(input): ValidatedForm<ContentStateInput>,
) -> Result<impl IntoResponse, HandlerError> {
    if !matches!(
        input.state.as_str(),
        "idea" | "draft" | "ready" | "posted_manual"
    ) {
        return Err(HandlerError::Deserialization(
            "invalid content state".into(),
        ));
    }
    db.update_distribution_content_state(group_id, content_id, &input.state)
        .await?;
    Ok((StatusCode::NO_CONTENT, REFRESH))
}

pub(crate) async fn download_csv(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    let data = db.get_distribution_dashboard(group_id).await?;
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer
        .write_record(["kind", "name", "channel", "code", "clicks", "registrations"])
        .map_err(anyhow::Error::from)?;
    for campaign in data.campaigns {
        writer
            .write_record([
                "campaign",
                &csv_safe(&campaign.name),
                "",
                "",
                &campaign.clicks.to_string(),
                &campaign.registrations.to_string(),
            ])
            .map_err(anyhow::Error::from)?;
    }
    for link in data.links {
        writer
            .write_record([
                "link",
                "",
                &csv_safe(&link.channel),
                &csv_safe(&link.code),
                &link.clicks.to_string(),
                &link.registrations.to_string(),
            ])
            .map_err(anyhow::Error::from)?;
    }
    for partner in data.partners {
        writer
            .write_record([
                "partner",
                &csv_safe(&partner.name),
                "",
                &csv_safe(&partner.referral_code),
                &partner.clicks.to_string(),
                &partner.registrations.to_string(),
            ])
            .map_err(anyhow::Error::from)?;
    }
    for channel in data.channel_metrics {
        writer
            .write_record([
                "channel",
                "",
                &csv_safe(&channel.channel),
                "",
                &channel.clicks.to_string(),
                &channel.registrations.to_string(),
            ])
            .map_err(anyhow::Error::from)?;
    }
    let body = writer
        .into_inner()
        .map_err(|error| anyhow::Error::from(error.into_error()))?;
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"distribution-metrics.csv\"",
            ),
        ],
        body,
    ))
}

#[cfg(test)]
mod tests {
    use super::csv_safe;

    #[test]
    fn csv_values_that_can_execute_formulas_are_escaped() {
        for value in [
            "=1+1", "+cmd", "-1+2", "@SUM(A1)", "\t=1+1", "\r@cmd", "\n-2+3",
        ] {
            assert!(csv_safe(value).starts_with('\''));
        }
        assert_eq!(csv_safe("Campaign"), "Campaign");
    }
}
