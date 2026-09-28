//! Shared GTM dashboard helpers used by alliance and group handlers.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Path, RawQuery, State},
    http::{HeaderName, StatusCode},
    response::{Html, IntoResponse},
};
use garde::Validate;
use serde_json::json;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{CurrentUser, ValidatedForm},
    },
    router::serde_qs_config,
    services::{
        gtm as gtm_service,
        notifications::{DynNotificationsManager, OutboundEmail},
    },
    templates::dashboard::gtm::{DetailPage, LeadGenerationDraft, ListPage},
    types::{
        gtm::{
            GtmActivityInput, GtmDeliverableInput, GtmLeadFilters, GtmLeadInput,
            GtmLeadTransitionInput, GtmReviewDraftInput, GtmRunAgentInput, GtmSponsorContactInput,
            GtmSponsorPackageInput, GtmSponsorProposalInput, GtmStateInput, GtmTaskInput,
        },
        pagination::{self, NavigationLinks},
        permissions::{AlliancePermission, GroupPermission},
    },
};

/// Alliance or group GTM surface.
#[derive(Clone, Copy)]
pub(crate) enum GtmScope {
    /// Alliance-wide pipeline.
    Alliance,
    /// Group-scoped pipeline.
    Group,
}

/// Creates a package in the selected campaign scope.
pub(crate) async fn add_package(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    alliance_id: Uuid,
    group_id: Option<Uuid>,
    ValidatedForm(mut input): ValidatedForm<GtmSponsorPackageInput>,
) -> Result<impl IntoResponse, HandlerError> {
    input.group_id = group_id;
    let deliverables = input
        .deliverables
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|name| json!({ "name": name, "quantity": 1 }))
        .collect::<Vec<_>>();
    let mut value = serde_json::to_value(&input)?;
    value["deliverables"] = json!(deliverables);
    db.add_gtm_campaign_record("package", user.user_id, alliance_id, None, &value)
        .await?;
    Ok((StatusCode::CREATED, [("HX-Trigger", "refresh-body")]).into_response())
}

macro_rules! add_lead_record_handler {
    ($name:ident, $input:ty, $record:literal) => {
        pub(crate) async fn $name(
            CurrentUser(user): CurrentUser,
            State(db): State<DynDB>,
            Path(lead_id): Path<Uuid>,
            alliance_id: Uuid,
            ValidatedForm(input): ValidatedForm<$input>,
        ) -> Result<impl IntoResponse, HandlerError> {
            db.add_gtm_campaign_record(
                $record,
                user.user_id,
                alliance_id,
                Some(lead_id),
                &serde_json::to_value(&input)?,
            )
            .await?;
            Ok((StatusCode::CREATED, [("HX-Trigger", "refresh-body")]).into_response())
        }
    };
}

add_lead_record_handler!(add_contact, GtmSponsorContactInput, "contact");
add_lead_record_handler!(add_proposal, GtmSponsorProposalInput, "proposal");
add_lead_record_handler!(add_task, GtmTaskInput, "task");
add_lead_record_handler!(add_deliverable, GtmDeliverableInput, "deliverable");

/// Adds an organizer-authored contact-history note.
pub(crate) async fn add_activity(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(lead_id): Path<Uuid>,
    alliance_id: Uuid,
    ValidatedForm(input): ValidatedForm<GtmActivityInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.add_gtm_lead_activity(user.user_id, alliance_id, lead_id, &input.kind, &input.body)
        .await?;
    Ok((StatusCode::CREATED, [("HX-Trigger", "refresh-body")]).into_response())
}

/// Changes a task or deliverable state.
pub(crate) async fn set_campaign_state(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(record_id): Path<Uuid>,
    alliance_id: Uuid,
    group_id: Option<Uuid>,
    record: &'static str,
    ValidatedForm(input): ValidatedForm<GtmStateInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.set_gtm_campaign_state(
        record,
        user.user_id,
        alliance_id,
        group_id,
        record_id,
        &input.state,
    )
    .await?;
    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-body")]).into_response())
}

impl GtmScope {
    fn dashboard_base(self) -> &'static str {
        match self {
            Self::Alliance => "/dashboard/alliance",
            Self::Group => "/dashboard/group",
        }
    }

    fn dashboard_tab_url(self) -> &'static str {
        match self {
            Self::Alliance => "/dashboard/alliance?tab=gtm",
            Self::Group => "/dashboard/group?tab=gtm",
        }
    }

    fn partial_url(self) -> &'static str {
        match self {
            Self::Alliance => "/dashboard/alliance/gtm",
            Self::Group => "/dashboard/group/gtm",
        }
    }
}

/// Lists GTM leads for the selected dashboard.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    RawQuery(raw_query): RawQuery,
    alliance_id: Uuid,
    group_id: Option<Uuid>,
    scope: GtmScope,
) -> Result<impl IntoResponse, HandlerError> {
    let (filters, template) = prepare_list_page(
        &db,
        alliance_id,
        user.user_id,
        group_id,
        scope,
        raw_query.as_deref().unwrap_or_default(),
    )
    .await?;
    let url = pagination::build_url(scope.dashboard_tab_url(), &filters)?;
    let headers = [(HeaderName::from_static("hx-push-url"), url)];
    Ok((headers, Html(template.render()?)))
}

/// Shows one lead.
#[instrument(skip_all, err)]
pub(crate) async fn detail_page(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(gtm_lead_id): Path<Uuid>,
    alliance_id: Uuid,
    group_id: Option<Uuid>,
    scope: GtmScope,
) -> Result<impl IntoResponse, HandlerError> {
    let template =
        prepare_detail_page(&db, alliance_id, user.user_id, group_id, scope, gtm_lead_id).await?;
    Ok(Html(template.render()?))
}

/// Creates a lead.
#[instrument(skip_all, err)]
pub(crate) async fn add(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    alliance_id: Uuid,
    group_id: Option<Uuid>,
    ValidatedForm(mut input): ValidatedForm<GtmLeadInput>,
) -> Result<impl IntoResponse, HandlerError> {
    if let Some(group_id) = group_id {
        input.group_id = Some(group_id);
    }
    if input.source.is_none() {
        input.source = Some("manual".to_string());
    }
    if let Some(currency) = &mut input.currency {
        currency.make_ascii_uppercase();
    }
    db.add_gtm_lead(user.user_id, alliance_id, &input).await?;
    Ok((StatusCode::CREATED, [("HX-Trigger", "refresh-body")]).into_response())
}

/// Updates a lead.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(gtm_lead_id): Path<Uuid>,
    alliance_id: Uuid,
    group_id: Option<Uuid>,
    ValidatedForm(mut input): ValidatedForm<GtmLeadInput>,
) -> Result<impl IntoResponse, HandlerError> {
    if let Some(group_id) = group_id {
        input.group_id = Some(group_id);
    }
    if let Some(currency) = &mut input.currency {
        currency.make_ascii_uppercase();
    }
    db.update_gtm_lead(
        user.user_id,
        alliance_id,
        gtm_lead_id,
        &serde_json::to_value(&input)?,
    )
    .await?;
    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-body")]).into_response())
}

/// Deletes a lead.
#[instrument(skip_all, err)]
pub(crate) async fn delete(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(gtm_lead_id): Path<Uuid>,
    alliance_id: Uuid,
    group_id: Option<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.delete_gtm_lead(user.user_id, alliance_id, gtm_lead_id, group_id)
        .await?;
    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-body")]).into_response())
}

/// Moves a lead to another stage.
#[instrument(skip_all, err)]
pub(crate) async fn transition(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(gtm_lead_id): Path<Uuid>,
    alliance_id: Uuid,
    ValidatedForm(input): ValidatedForm<GtmLeadTransitionInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.transition_gtm_lead(
        user.user_id,
        alliance_id,
        gtm_lead_id,
        &input.stage,
        true,
        &json!({ "lost_reason": input.lost_reason }),
    )
    .await?;
    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-body")]).into_response())
}

/// Runs a draft-only agent.
#[instrument(skip_all, err)]
pub(crate) async fn run_agent(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    alliance_id: Uuid,
    group_id: Option<Uuid>,
    gtm_lead_id: Option<Uuid>,
    ValidatedForm(input): ValidatedForm<GtmRunAgentInput>,
) -> Result<impl IntoResponse, HandlerError> {
    if input.agent_id == "lead_generation" && gtm_lead_id.is_none() {
        let drafts = db
            .list_gtm_agent_drafts(
                alliance_id,
                &json!({
                    "agent_id": "lead_generation",
                    "status": "pending",
                    "group_id": group_id,
                    "exact_scope": true
                }),
            )
            .await?;
        if !drafts.drafts.is_empty() {
            return Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-body")]).into_response());
        }
    }
    gtm_service::run_agent(
        &db,
        user.user_id,
        alliance_id,
        group_id,
        gtm_lead_id,
        &input.agent_id,
        input.reply.as_deref(),
    )
    .await?;
    Ok((StatusCode::CREATED, [("HX-Trigger", "refresh-body")]).into_response())
}

/// Approves or rejects a draft.
#[instrument(skip_all, err)]
pub(crate) async fn review_draft(
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    State(notifications_manager): State<DynNotificationsManager>,
    Path(draft_id): Path<Uuid>,
    alliance_id: Uuid,
    group_id: Option<Uuid>,
    ValidatedForm(input): ValidatedForm<GtmReviewDraftInput>,
) -> Result<impl IntoResponse, HandlerError> {
    let scope_drafts = db
        .list_gtm_agent_drafts(
            alliance_id,
            &json!({
                "status": "pending",
                "group_id": group_id,
                "exact_scope": true
            }),
        )
        .await?;
    if !scope_drafts
        .drafts
        .iter()
        .any(|draft| draft.gtm_agent_draft_id == draft_id)
    {
        return Err(HandlerError::NotFound);
    }

    let mut payload = json!({});
    if input.create_sponsor || input.create_landscape_entry || input.invite_organizer {
        payload["side_effects"] = json!({
            "create_sponsor": input.create_sponsor,
            "create_landscape_entry": input.create_landscape_entry,
            "invite_organizer": input.invite_organizer
        });
    }
    let result = db
        .review_gtm_agent_draft(
            user.user_id,
            alliance_id,
            draft_id,
            &json!({
                "status": input.status,
                "body": input.body,
                "suggested_stage": input.suggested_stage,
                "note": input.note,
                "payload": payload
            }),
        )
        .await?;

    if input.status == "approved"
        && matches!(result.agent_id.as_deref(), Some("reachout" | "renewal"))
        && let Some(lead_id) = result.gtm_lead_id
        && let Some(lead) = db.get_gtm_lead(alliance_id, lead_id).await?
        && let Some(email) = lead.email.clone()
    {
        let subject = match result.agent_id.as_deref() {
            Some("renewal") => format!("Renewal: {}", lead.name),
            _ => format!(
                "Partnership with {}",
                lead.org_name.as_deref().unwrap_or(&lead.name)
            ),
        };
        notifications_manager
            .send_email(&OutboundEmail {
                body: result.body.clone().unwrap_or_default(),
                subject,
                to: email,
            })
            .await?;
    }

    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-body")]).into_response())
}

/// Prepares the list page used by dashboard tabs and HTMX partials.
pub(crate) async fn prepare_list_page(
    db: &DynDB,
    alliance_id: Uuid,
    user_id: Uuid,
    group_id: Option<Uuid>,
    scope: GtmScope,
    raw_query: &str,
) -> Result<(GtmLeadFilters, ListPage), HandlerError> {
    let mut filters: GtmLeadFilters = if raw_query.is_empty() {
        GtmLeadFilters::default()
    } else {
        serde_qs_config().deserialize_str(raw_query)?
    };
    if filters.group_id.is_none() {
        filters.group_id = group_id;
    }
    filters.validate()?;

    let lead_generation_filters = json!({
        "agent_id": "lead_generation",
        "status": "pending",
        "group_id": group_id,
        "exact_scope": true
    });
    let (can_manage_gtm, output, packages, due_tasks, lead_generation_drafts) = tokio::try_join!(
        user_can_manage_gtm(db, alliance_id, group_id, user_id, scope),
        db.list_gtm_leads(alliance_id, &filters),
        db.list_gtm_sponsor_packages(alliance_id, group_id),
        db.list_due_gtm_tasks(alliance_id, group_id),
        db.list_gtm_agent_drafts(alliance_id, &lead_generation_filters)
    )?;
    let lead_generation_draft = prepare_lead_generation_draft(lead_generation_drafts.drafts);
    let navigation_links = NavigationLinks::from_filters(
        &filters,
        output.total,
        scope.dashboard_tab_url(),
        scope.partial_url(),
    )?;

    Ok((
        filters.clone(),
        ListPage {
            can_manage_gtm,
            dashboard_base: scope.dashboard_base().to_string(),
            dashboard_tab_url: scope.dashboard_tab_url().to_string(),
            filters,
            leads: output.leads,
            total: output.total,
            stage_counts: output.stage_counts,
            navigation_links,
            packages: packages.0,
            due_tasks: due_tasks.0,
            lead_generation_draft,
        },
    ))
}

fn prepare_lead_generation_draft(
    mut drafts: Vec<crate::types::gtm::GtmAgentDraft>,
) -> Option<LeadGenerationDraft> {
    let older_pending_count = drafts.len().saturating_sub(1);
    let draft = drafts.drain(..).next()?;
    let candidates = draft
        .payload
        .get("candidates")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|candidate| serde_json::from_value(candidate.clone()).ok())
        .collect();
    Some(LeadGenerationDraft {
        draft,
        candidates,
        older_pending_count,
    })
}

async fn prepare_detail_page(
    db: &DynDB,
    alliance_id: Uuid,
    user_id: Uuid,
    group_id: Option<Uuid>,
    scope: GtmScope,
    gtm_lead_id: Uuid,
) -> Result<DetailPage, HandlerError> {
    let draft_filters = json!({ "gtm_lead_id": gtm_lead_id });
    let (can_manage_gtm, lead, drafts, packages) = tokio::try_join!(
        user_can_manage_gtm(db, alliance_id, group_id, user_id, scope),
        db.get_gtm_lead(alliance_id, gtm_lead_id),
        db.list_gtm_agent_drafts(alliance_id, &draft_filters),
        db.list_gtm_sponsor_packages(alliance_id, group_id)
    )?;
    let Some(mut lead) = lead else {
        return Err(HandlerError::NotFound);
    };
    if let Some(expected_group) = group_id
        && lead.group_id != Some(expected_group)
    {
        return Err(HandlerError::NotFound);
    }
    lead.drafts = drafts.drafts;
    Ok(DetailPage {
        can_manage_gtm,
        dashboard_base: scope.dashboard_base().to_string(),
        dashboard_tab_url: scope.dashboard_tab_url().to_string(),
        lead,
        packages: packages.0,
    })
}

async fn user_can_manage_gtm(
    db: &DynDB,
    alliance_id: Uuid,
    group_id: Option<Uuid>,
    user_id: Uuid,
    scope: GtmScope,
) -> Result<bool> {
    match scope {
        GtmScope::Alliance => {
            db.user_has_alliance_permission(&alliance_id, &user_id, AlliancePermission::GtmWrite)
                .await
        }
        GtmScope::Group => {
            let Some(group_id) = group_id else {
                return Ok(false);
            };
            db.user_has_group_permission(
                &alliance_id,
                &group_id,
                &user_id,
                GroupPermission::GtmWrite,
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    use super::prepare_lead_generation_draft;
    use crate::types::gtm::GtmAgentDraft;

    #[test]
    fn prepares_latest_lead_generation_draft_candidates() {
        let latest_id = Uuid::new_v4();
        let drafts = vec![
            GtmAgentDraft {
                gtm_agent_draft_id: latest_id,
                agent_id: "lead_generation".into(),
                status: "pending".into(),
                payload: json!({
                    "candidates": [{
                        "name": "Candidate One",
                        "kind": "startup",
                        "org_name": "Candidate Co",
                        "website_url": "https://candidate.example"
                    }]
                }),
                created_at: Utc::now(),
                ..Default::default()
            },
            GtmAgentDraft {
                gtm_agent_draft_id: Uuid::new_v4(),
                agent_id: "lead_generation".into(),
                status: "pending".into(),
                created_at: Utc::now(),
                ..Default::default()
            },
        ];

        let review = prepare_lead_generation_draft(drafts).expect("latest draft");

        assert_eq!(review.draft.gtm_agent_draft_id, latest_id);
        assert_eq!(review.older_pending_count, 1);
        assert_eq!(review.candidates.len(), 1);
        assert_eq!(review.candidates[0].name, "Candidate One");
        assert_eq!(review.candidates[0].kind, "startup");
    }
}
