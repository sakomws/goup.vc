//! HTTP handlers for managing events in the group dashboard.

use std::collections::HashMap;

use anyhow::Result;
use askama::Template;
use axum::{
    Json,
    extract::{Path, Query, RawQuery, State},
    http::{
        HeaderName, StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    },
    response::{Html, IntoResponse},
};
use chrono::{NaiveDate, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    config::{HttpServerConfig, MeetingsConfig, PaymentsConfig},
    db::{DBExt, DBOperations, DynDB},
    handlers::{
        error::HandlerError,
        extractors::{CurrentUser, SelectedAllianceId, SelectedGroupId, ValidatedFormQs},
    },
    router::serde_qs_config,
    services::{
        meetings::MeetingProvider,
        notifications::enqueue::{
            enqueue_event_canceled_notification, enqueue_event_published_notifications,
            enqueue_event_rescheduled_notification, enqueue_event_series_canceled_notifications,
            enqueue_event_series_published_notifications,
            enqueue_event_waitlist_promoted_notification,
        },
    },
    templates::dashboard::group::{
        events::{self, Event, EventsListFilters, EventsTab},
        sponsors::GroupSponsorsFilters,
    },
    templates::event::SponsorReportPage,
    types::{
        event::{EventGrowth, EventSummary},
        pagination::{self, NavigationLinks},
        payments::GroupPaymentRecipient,
        permissions::{AlliancePermission, GroupPermission},
    },
    validation::{MAX_LEN_DESCRIPTION_SHORT, MAX_LEN_S, trimmed_non_empty, trimmed_non_empty_opt},
};

mod recurrence;

#[cfg(test)]
mod tests;

use recurrence::RecurringEventPayloads;

// URLs used by the dashboard page and tab partial
const DASHBOARD_URL: &str = "/dashboard/group?tab=events";
const PARTIAL_URL: &str = "/dashboard/group/events";

// Pages handlers.

/// Displays the page to add a new event.
#[instrument(skip_all, err)]
pub(crate) async fn add_page(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    State(meetings_cfg): State<Option<MeetingsConfig>>,
    State(payments_cfg): State<Option<PaymentsConfig>>,
) -> Result<impl IntoResponse, HandlerError> {
    // Fetch template data concurrently
    let meetings_enabled = meetings_cfg.as_ref().is_some_and(MeetingsConfig::meetings_enabled);
    let meetings_max_participants = build_meetings_max_participants(meetings_cfg.as_ref());
    let sponsor_filters: GroupSponsorsFilters = serde_qs_config().deserialize_str("")?;
    let (
        can_manage_events,
        categories,
        event_kinds,
        event_defaults,
        payment_currency_codes,
        payment_recipient,
        session_kinds,
        sponsors,
        timezones,
    ) = tokio::try_join!(
        db.user_has_group_permission(
            &alliance_id,
            &group_id,
            &user.user_id,
            GroupPermission::EventsWrite
        ),
        db.list_event_categories(alliance_id),
        db.list_event_kinds(),
        db.get_group_event_defaults(alliance_id, group_id),
        db.list_payment_currency_codes(),
        db.get_group_payment_recipient(alliance_id, group_id),
        db.list_session_kinds(),
        db.list_group_sponsors(group_id, &sponsor_filters, true),
        db.list_timezones()
    )?;

    // Prepare template
    let template = events::AddPage {
        can_manage_events,
        categories,
        event_kinds,
        event_defaults,
        group_id,
        meetings_enabled,
        payments_enabled: payments_cfg.is_some(),
        payment_currency_codes,
        payments_ready: payments_ready(payment_recipient.as_ref(), payments_cfg.as_ref()),
        meetings_max_participants,
        session_kinds,
        sponsors: sponsors.sponsors,
        timezones,
    };

    Ok(Html(template.render()?))
}

/// Displays the list of events for the group dashboard.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    // Prepare list page content
    let (filters, template) = prepare_list_page(
        &db,
        alliance_id,
        group_id,
        user.user_id,
        raw_query.as_deref().unwrap_or_default(),
    )
    .await?;

    // Prepare response headers
    let url = pagination::build_url(DASHBOARD_URL, &filters)?;
    let headers = [(HeaderName::from_static("hx-push-url"), url)];

    Ok((headers, Html(template.render()?)))
}

/// Renders a database-free preview from the submitted event editor state.
#[instrument(skip_all, err)]
pub(crate) async fn preview(
    State(serde_qs_de): State<serde_qs::Config>,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Prepare template
    let input: events::preview::Input = serde_qs_de
        .deserialize_str(&body)
        .map_err(|err| HandlerError::Deserialization(err.to_string()))?;
    let template = events::preview::Page {
        event: input.into(),
    };

    Ok(Html(template.render()?))
}

/// Displays the page to update an existing event.
#[instrument(skip_all, err)]
pub(crate) async fn update_page(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    State(meetings_cfg): State<Option<MeetingsConfig>>,
    State(payments_cfg): State<Option<PaymentsConfig>>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Prepare template
    let meetings_enabled = meetings_cfg.as_ref().is_some_and(MeetingsConfig::meetings_enabled);
    let meetings_max_participants = build_meetings_max_participants(meetings_cfg.as_ref());
    let sponsor_filters: GroupSponsorsFilters = serde_qs_config().deserialize_str("")?;
    let (
        can_manage_events,
        can_move_events,
        event,
        approved_submissions,
        categories,
        cfs_statuses,
        event_kinds,
        payment_currency_codes,
        payment_recipient,
        session_kinds,
        sponsors,
        timezones,
    ) = tokio::try_join!(
        db.user_has_group_permission(
            &alliance_id,
            &group_id,
            &user.user_id,
            GroupPermission::EventsWrite
        ),
        db.user_has_alliance_permission(
            &alliance_id,
            &user.user_id,
            AlliancePermission::GroupsWrite
        ),
        db.get_event_full(alliance_id, group_id, event_id),
        db.list_event_approved_cfs_submissions(event_id),
        db.list_event_categories(alliance_id),
        db.list_cfs_submission_statuses_for_review(),
        db.list_event_kinds(),
        db.list_payment_currency_codes(),
        db.get_group_payment_recipient(alliance_id, group_id),
        db.list_session_kinds(),
        db.list_group_sponsors(group_id, &sponsor_filters, true),
        db.list_timezones(),
    )?;
    let template = events::UpdatePage {
        approved_submissions,
        can_manage_events,
        can_move_events,
        categories,
        cfs_submission_statuses: cfs_statuses,
        current_user_id: user.user_id,
        event,
        event_kinds,
        group_id,
        meetings_enabled,
        payments_enabled: payments_cfg.is_some(),
        payment_currency_codes,
        payments_ready: payments_ready(payment_recipient.as_ref(), payments_cfg.as_ref()),
        meetings_max_participants,
        session_kinds,
        sponsors: sponsors.sponsors,
        timezones,
    };

    Ok(Html(template.render()?))
}

// JSON handlers.

/// Returns full event details in JSON format.
#[instrument(skip_all, err)]
pub(crate) async fn details(
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let event = db.get_event_full(alliance_id, group_id, event_id).await?;

    Ok(Json(event).into_response())
}

/// Renders per-event funnel, attribution, collaboration, and finance analytics.
#[instrument(skip_all, err)]
pub(crate) async fn growth_page(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    render_growth_page(&db, alliance_id, group_id, event_id, user.user_id).await
}

/// Adds a categorized manual event income or expense.
#[instrument(skip_all, err)]
pub(crate) async fn add_finance_entry(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
    ValidatedFormQs(entry): ValidatedFormQs<EventFinanceEntryInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.add_event_finance_entry(
        user.user_id,
        group_id,
        event_id,
        &serde_json::to_value(entry)
            .map_err(|err| HandlerError::Deserialization(err.to_string()))?,
    )
    .await?;

    let growth = db.get_event_growth(group_id, event_id).await?;
    Ok(Html(
        events::GrowthPage {
            can_manage_events: true,
            event_id,
            growth,
        }
        .render()?,
    ))
}

/// Deletes a manual event finance entry.
#[instrument(skip_all, err)]
pub(crate) async fn delete_finance_entry(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path((event_id, entry_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, HandlerError> {
    db.delete_event_finance_entry(group_id, event_id, entry_id).await?;
    let growth = db.get_event_growth(group_id, event_id).await?;
    Ok(Html(
        events::GrowthPage {
            can_manage_events: true,
            event_id,
            growth,
        }
        .render()?,
    ))
}

/// Updates organizer-entered, consented aggregate sponsor engagement counts.
#[instrument(skip_all, err)]
pub(crate) async fn update_sponsor_engagement(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path((event_id, group_sponsor_id)): Path<(Uuid, Uuid)>,
    ValidatedFormQs(input): ValidatedFormQs<EventSponsorEngagementInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.update_event_sponsor_manual_engagement(
        user.user_id,
        group_id,
        event_id,
        group_sponsor_id,
        &serde_json::to_value(input)
            .map_err(|err| HandlerError::Deserialization(err.to_string()))?,
    )
    .await?;
    let growth = db.get_event_growth(group_id, event_id).await?;
    Ok(Html(
        events::GrowthPage {
            can_manage_events: true,
            event_id,
            growth,
        }
        .render()?,
    ))
}

/// Renders the organizer-only sponsor report preview.
#[instrument(skip_all, err)]
pub(crate) async fn sponsor_report_preview(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let growth = db.get_event_growth(group_id, event_id).await?;
    let report = growth.sponsor_report.ok_or(HandlerError::NotFound)?;
    Ok((
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        Html(
            SponsorReportPage {
                report,
                public: false,
            }
            .render()?,
        ),
    ))
}

/// Lists aggregate survey metrics and anonymous response review rows.
#[instrument(skip_all, err)]
pub(crate) async fn surveys_page(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
    Query(filters): Query<EventSurveyFilters>,
) -> Result<impl IntoResponse, HandlerError> {
    validate_survey_audience(filters.audience.as_deref())?;
    let dashboard = db
        .get_event_survey_dashboard(group_id, event_id, filters.audience.clone())
        .await?;
    Ok((
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        Html(
            events::SurveyDashboardPage {
                event_id,
                audience: filters.audience,
                dashboard,
            }
            .render()?,
        ),
    ))
}

/// Exports aggregate survey rows without respondent identity.
#[instrument(skip_all, err)]
pub(crate) async fn download_surveys_csv(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
    Query(filters): Query<EventSurveyFilters>,
) -> Result<impl IntoResponse, HandlerError> {
    validate_survey_audience(filters.audience.as_deref())?;
    let dashboard = db
        .get_event_survey_dashboard(group_id, event_id, filters.audience.clone())
        .await?;
    let csv = (|| -> anyhow::Result<String> {
        let mut writer = csv::Writer::from_writer(Vec::new());
        writer.write_record([
            "row_type",
            "audience",
            "submitted_at",
            "nps",
            "rating",
            "anonymous_comment",
            "eligible",
            "responses",
            "response_rate",
            "promoters",
            "passives",
            "detractors",
            "nps_score",
            "average_rating",
        ])?;
        for metric in dashboard.metrics {
            writer.write_record([
                "aggregate".to_string(),
                metric.audience,
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                metric.eligible.to_string(),
                metric.responses.to_string(),
                metric.response_rate.to_string(),
                metric.promoters.to_string(),
                metric.passives.to_string(),
                metric.detractors.to_string(),
                metric.nps_score.map(|value| value.to_string()).unwrap_or_default(),
                metric
                    .average_rating
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            ])?;
        }
        for response in dashboard.responses {
            writer.write_record([
                "response".to_string(),
                response.audience,
                response.submitted_at.to_rfc3339(),
                response.nps.map(|value| value.to_string()).unwrap_or_default(),
                response.rating.map(|value| value.to_string()).unwrap_or_default(),
                spreadsheet_safe_text(response.comment.as_deref()),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ])?;
        }
        Ok(String::from_utf8(writer.into_inner()?)?)
    })()?;
    Ok((
        [
            (CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                CONTENT_DISPOSITION,
                format!("attachment; filename=\"event-{event_id}-surveys.csv\""),
            ),
            (
                axum::http::header::CACHE_CONTROL,
                "private, no-store".to_string(),
            ),
        ],
        csv,
    ))
}

/// Optional dashboard audience filter.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct EventSurveyFilters {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    audience: Option<String>,
}

fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value.filter(|value| !value.trim().is_empty()))
}

fn validate_survey_audience(audience: Option<&str>) -> Result<(), HandlerError> {
    if audience.is_none_or(|value| matches!(value, "attendee" | "speaker" | "sponsor-contact")) {
        Ok(())
    } else {
        Err(HandlerError::Deserialization(
            "invalid survey audience".to_string(),
        ))
    }
}

/// Prevents spreadsheet applications from evaluating anonymous comments as formulas.
fn spreadsheet_safe_text(value: Option<&str>) -> String {
    let value = value.unwrap_or_default();
    if value.starts_with(['=', '+', '-', '@']) {
        format!("'{value}")
    } else {
        value.to_string()
    }
}

/// Creates or rotates the public sponsor report link.
#[instrument(skip_all, err)]
pub(crate) async fn create_sponsor_report_share(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    State(server_cfg): State<HttpServerConfig>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let token = db
        .create_event_sponsor_report_share(user.user_id, group_id, event_id)
        .await?;
    let url = format!(
        "{}/sponsor-reports/{token}",
        server_cfg.base_url.trim_end_matches('/')
    );
    Ok(Html(format!(
        r#"<a class="text-sm text-primary-700 underline" href="{url}" target="_blank" rel="noopener noreferrer">{url}</a>"#
    )))
}

/// Revokes the current public sponsor report link.
#[instrument(skip_all, err)]
pub(crate) async fn revoke_sponsor_report_share(
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.revoke_event_sponsor_report_share(group_id, event_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Downloads the event growth report as CSV.
#[instrument(skip_all, err)]
pub(crate) async fn download_growth_csv(
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let (event, growth) = tokio::try_join!(
        db.get_event_summary(alliance_id, group_id, event_id),
        db.get_event_growth(group_id, event_id)
    )?;
    let csv = build_growth_csv(&growth)?;
    let file_name = format!("event-{}-growth.csv", event.slug);

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

/// Stores the current event details as the default template for new group events.
#[instrument(skip_all, err)]
pub(crate) async fn set_group_defaults(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let event = db.get_event_full(alliance_id, group_id, event_id).await?;
    let event_defaults = sanitize_event_defaults(
        serde_json::to_value(event)
            .map_err(|err| HandlerError::Deserialization(err.to_string()))?,
    );
    db.update_group_event_defaults(user.user_id, group_id, event_defaults)
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "group-event-defaults-updated")],
    )
        .into_response())
}

/// Returns the `<option>` list of groups the current event can be moved into.
#[instrument(skip_all, err)]
pub(crate) async fn move_targets(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(_event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let targets = db
        .list_group_move_targets(user.user_id, alliance_id, group_id)
        .await?;
    let template = events::MoveTargets { targets };

    Ok(Html(template.render()?))
}

/// Payload for moving an event to another group.
#[derive(Debug, Deserialize, Validate)]
pub(crate) struct MoveEventForm {
    /// Target group the event should be moved into.
    #[garde(skip)]
    pub target_group_id: Uuid,
}

/// Moves an event to another group within the same alliance.
#[instrument(skip_all, err)]
pub(crate) async fn move_to_group(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
    ValidatedFormQs(form): ValidatedFormQs<MoveEventForm>,
) -> Result<impl IntoResponse, HandlerError> {
    db.move_event(user.user_id, group_id, event_id, form.target_group_id)
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [(
            "HX-Location",
            r#"{"path":"/dashboard/group?tab=events", "target":"body"}"#,
        )],
    ))
}

// Actions handlers.

/// Adds a new event to the database.
#[instrument(skip_all, err)]
pub(crate) async fn add(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    State(meetings_cfg): State<Option<MeetingsConfig>>,
    State(payments_cfg): State<Option<crate::config::PaymentsConfig>>,
    ValidatedFormQs(event): ValidatedFormQs<Event>,
) -> Result<impl IntoResponse, HandlerError> {
    // Prepare and validate the event payload
    let cfg_max_participants = build_meetings_max_participants(meetings_cfg.as_ref());
    let event_payload = build_event_payload(&event)?;
    if event_payload_uses_ticketing(&event_payload) {
        ensure_ticketing_ready(&db, alliance_id, group_id, payments_cfg.as_ref()).await?;
    }

    // Create either a single event or a linked recurring event series
    if let Some(recurring_event_payloads) =
        RecurringEventPayloads::from_event(&event, &event_payload)
            .map_err(|err| HandlerError::Deserialization(err.to_string()))?
    {
        db.add_event_series(
            user.user_id,
            group_id,
            &recurring_event_payloads.events,
            &recurring_event_payloads.recurrence,
            &cfg_max_participants,
        )
        .await?;
    } else {
        db.add_event(
            user.user_id,
            group_id,
            &event_payload,
            &cfg_max_participants,
        )
        .await?;
    }

    Ok((
        StatusCode::CREATED,
        [("HX-Trigger", "refresh-group-dashboard-table")],
    )
        .into_response())
}

/// Cancels an event (sets canceled=true).
#[allow(clippy::too_many_arguments)]
#[instrument(skip_all, err)]
pub(crate) async fn cancel(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    State(server_cfg): State<HttpServerConfig>,
    Path(event_id): Path<Uuid>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    // Resolve action scope
    let query = parse_event_action_query(raw_query.as_deref())?;
    let scope = query.scope;

    db.as_ref()
        .transaction(|tx| {
            Box::pin(async move {
                // Load summaries before canceling so notification eligibility uses prior state
                let event_ids = event_action_ids(tx, group_id, event_id, scope).await?;
                let mut events = Vec::with_capacity(event_ids.len());
                for event_id in &event_ids {
                    events.push(tx.get_event_summary(alliance_id, group_id, *event_id).await?);
                }

                // Mark the selected event or the whole linked series as canceled
                match scope {
                    EventActionScope::Series => {
                        tx.cancel_event_series_events(user.user_id, group_id, &event_ids)
                            .await?;
                    }
                    EventActionScope::This => {
                        tx.cancel_event(user.user_id, group_id, event_id).await?;
                    }
                }

                // Enqueue cancellation notifications for future published events
                let events_to_notify: Vec<EventSummary> = events
                    .into_iter()
                    .filter(|event| {
                        matches!(
                            (event.published, event.canceled, event.starts_at),
                            (true, false, Some(starts_at))
                                if !event.test_event && starts_at > Utc::now()
                        )
                    })
                    .collect();
                match (scope, events_to_notify.as_slice()) {
                    // Multiple notifiable events
                    (EventActionScope::Series, [_, _, ..]) => {
                        let event_ids: Vec<Uuid> =
                            events_to_notify.iter().map(|event| event.event_id).collect();
                        enqueue_event_series_canceled_notifications(
                            tx,
                            &server_cfg,
                            alliance_id,
                            group_id,
                            &event_ids,
                        )
                        .await?;
                    }
                    // Single notifiable event
                    (_, [event]) => {
                        enqueue_event_canceled_notification(
                            tx,
                            &server_cfg,
                            alliance_id,
                            group_id,
                            event.event_id,
                        )
                        .await?;
                    }
                    _ => {}
                }

                Ok(())
            })
        })
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [(
            "HX-Location",
            r#"{"path":"/dashboard/group?tab=events", "target":"body"}"#,
        )],
    ))
}

/// Deletes an event from the database (soft delete).
#[instrument(skip_all, err)]
pub(crate) async fn delete(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    // Resolve action scope
    let query = parse_event_action_query(raw_query.as_deref())?;

    // Delete the selected event or the whole linked series
    match query.scope {
        EventActionScope::Series => {
            let event_ids = event_action_ids(db.as_ref(), group_id, event_id, query.scope).await?;
            db.delete_event_series_events(user.user_id, group_id, &event_ids)
                .await?;
        }
        EventActionScope::This => db.delete_event(user.user_id, group_id, event_id).await?,
    }

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-group-dashboard-table")],
    ))
}

/// Publishes an event (sets published=true and records publication metadata).
#[allow(clippy::too_many_arguments)]
#[instrument(skip_all, err)]
pub(crate) async fn publish(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    State(payments_cfg): State<Option<PaymentsConfig>>,
    State(server_cfg): State<HttpServerConfig>,
    Path(event_id): Path<Uuid>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    // Resolve action scope
    let query = parse_event_action_query(raw_query.as_deref())?;
    let scope = query.scope;
    let configured_provider = payments_cfg.as_ref().map(PaymentsConfig::provider);

    db.as_ref()
        .transaction(|tx| {
            Box::pin(async move {
                // Resolve target event ids and load prior state before publishing
                let event_ids = match scope {
                    EventActionScope::Series => {
                        tx.list_event_series_publishable_event_ids(group_id, event_id).await?
                    }
                    EventActionScope::This => vec![event_id],
                };
                let mut events = Vec::with_capacity(event_ids.len());
                for event_id in &event_ids {
                    events.push(tx.get_event_summary(alliance_id, group_id, *event_id).await?);
                }

                // Publish the selected event or the whole linked series
                match scope {
                    EventActionScope::Series => {
                        tx.publish_event_series_events(
                            user.user_id,
                            configured_provider,
                            group_id,
                            &event_ids,
                        )
                        .await?;
                    }
                    EventActionScope::This => {
                        tx.publish_event(user.user_id, configured_provider, group_id, event_id)
                            .await?;
                    }
                }

                // Enqueue required publish notifications before committing
                let events_to_notify: Vec<EventSummary> = events
                    .into_iter()
                    .filter(|event| {
                        matches!(
                            (event.published, event.starts_at),
                            (false, Some(starts_at)) if !event.test_event && starts_at > Utc::now()
                        )
                    })
                    .collect();
                match (scope, events_to_notify.as_slice()) {
                    // Multiple notifiable events
                    (EventActionScope::Series, [_, _, ..]) => {
                        let event_ids: Vec<Uuid> =
                            events_to_notify.iter().map(|event| event.event_id).collect();
                        enqueue_event_series_published_notifications(
                            tx,
                            &server_cfg,
                            alliance_id,
                            group_id,
                            &event_ids,
                        )
                        .await?;
                    }
                    // Single notifiable event
                    (_, [event]) => {
                        enqueue_event_published_notifications(
                            tx,
                            &server_cfg,
                            alliance_id,
                            group_id,
                            event.event_id,
                        )
                        .await?;
                    }
                    _ => {}
                }

                Ok(())
            })
        })
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-group-dashboard-table")],
    ))
}

/// Unpublishes an event (sets published=false and clears publication metadata).
#[instrument(skip_all, err)]
pub(crate) async fn unpublish(
    CurrentUser(user): CurrentUser,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    // Resolve action scope
    let query = parse_event_action_query(raw_query.as_deref())?;

    // Unpublish the selected event or the whole linked series
    match query.scope {
        EventActionScope::Series => {
            let event_ids = event_action_ids(db.as_ref(), group_id, event_id, query.scope).await?;
            db.unpublish_event_series_events(user.user_id, group_id, &event_ids)
                .await?;
        }
        EventActionScope::This => db.unpublish_event(user.user_id, group_id, event_id).await?,
    }

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-group-dashboard-table")],
    ))
}

/// Updates an existing event's information in the database.
#[allow(clippy::too_many_arguments)]
#[instrument(skip_all, err)]
pub(crate) async fn update(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    State(meetings_cfg): State<Option<MeetingsConfig>>,
    State(payments_cfg): State<Option<crate::config::PaymentsConfig>>,
    State(serde_qs_de): State<serde_qs::Config>,
    State(server_cfg): State<HttpServerConfig>,
    Path(event_id): Path<Uuid>,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Deserialize and validate provided event
    let event: Event = serde_qs_de
        .deserialize_str(&body)
        .map_err(|e| HandlerError::Deserialization(e.to_string()))?;
    event.validate()?;

    // Prepare update payload and ticketing prerequisites
    let cfg_max_participants = build_meetings_max_participants(meetings_cfg.as_ref());
    let event_json = build_event_payload(&event)?;
    if event_payload_uses_ticketing(&event_json) {
        ensure_ticketing_ready(&db, alliance_id, group_id, payments_cfg.as_ref()).await?;
    }

    db.as_ref()
        .transaction(|tx| {
            Box::pin(async move {
                // Load prior state before mutating to drive notification decisions
                let before = tx.get_event_summary(alliance_id, group_id, event_id).await?;

                // Update event in database
                let promoted_user_ids = tx
                    .update_event(
                        user.user_id,
                        group_id,
                        event_id,
                        &event_json,
                        &cfg_max_participants,
                    )
                    .await?;

                // Enqueue required waitlist promotion notifications before committing
                enqueue_event_waitlist_promoted_notification(
                    tx,
                    &server_cfg,
                    alliance_id,
                    group_id,
                    event_id,
                    &before,
                    promoted_user_ids,
                )
                .await?;

                // Enqueue required reschedule notifications before committing
                enqueue_event_rescheduled_notification(
                    tx,
                    &server_cfg,
                    alliance_id,
                    group_id,
                    event_id,
                    &before,
                )
                .await?;

                Ok(())
            })
        })
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-group-dashboard-table")],
    )
        .into_response())
}

// Types.

/// Query parameters accepted by cancel/delete actions.
#[derive(Debug, Default, Deserialize)]
struct EventActionQuery {
    /// Selected action scope.
    #[serde(default)]
    scope: EventActionScope,
}

/// Event management action scope requested by the dashboard.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum EventActionScope {
    /// Apply the action to the linked event series.
    Series,
    /// Apply the action only to the selected event.
    #[default]
    This,
}

/// Organizer-provided non-purchase event ledger row.
#[derive(Debug, Deserialize, Serialize, Validate)]
pub(crate) struct EventFinanceEntryInput {
    /// Positive amount in the currency's minor unit.
    #[garde(range(min = 1))]
    amount_minor: i64,
    /// Organizer-defined reporting category.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_S))]
    category: String,
    /// ISO 4217 currency code.
    #[garde(custom(trimmed_non_empty), length(min = 3, max = 3))]
    currency_code: String,
    /// Optional ledger detail.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION_SHORT))]
    description: Option<String>,
    /// `income` or `expense`.
    #[garde(custom(validate_finance_kind))]
    kind: String,
    /// Business date for the entry.
    #[garde(skip)]
    occurred_at: Option<NaiveDate>,
}

/// Consented/manual aggregate sponsor engagement. No lead identities are accepted.
#[derive(Debug, Deserialize, Serialize, Validate)]
pub(crate) struct EventSponsorEngagementInput {
    #[garde(range(min = 0))]
    conversations_count: i32,
    #[garde(range(min = 0))]
    leads_count: i32,
    #[garde(range(min = 0))]
    meetings_count: i32,
    #[garde(custom(trimmed_non_empty_opt), length(max = 2000))]
    notes: Option<String>,
}

// Helpers.

fn validate_finance_kind(value: &str, _context: &()) -> garde::Result {
    if matches!(value, "income" | "expense") {
        Ok(())
    } else {
        Err(garde::Error::new("must be income or expense"))
    }
}

async fn render_growth_page(
    db: &DynDB,
    alliance_id: Uuid,
    group_id: Uuid,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<Html<String>, HandlerError> {
    let (can_manage_events, growth) = tokio::try_join!(
        db.user_has_group_permission(
            &alliance_id,
            &group_id,
            &user_id,
            GroupPermission::EventsWrite
        ),
        db.get_event_growth(group_id, event_id)
    )?;
    Ok(Html(
        events::GrowthPage {
            can_manage_events,
            event_id,
            growth,
        }
        .render()?,
    ))
}

fn build_growth_csv(growth: &EventGrowth) -> Result<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(["section", "label", "currency", "value"])?;
    let funnel = &growth.funnel;
    for (label, value) in [
        ("views", funnel.views.to_string()),
        ("registrations", funnel.registrations.to_string()),
        ("waitlisted", funnel.waitlisted.to_string()),
        ("pending", funnel.pending.to_string()),
        ("confirmed", funnel.confirmed.to_string()),
        ("check_ins", funnel.check_ins.to_string()),
        (
            "conversion_rate_percent",
            funnel.conversion_rate.to_string(),
        ),
        ("unique_attendees", funnel.unique_attendees.to_string()),
        ("repeat_attendees", funnel.repeat_attendees.to_string()),
        ("new_members", funnel.new_members.to_string()),
        (
            "follow_up_collaborators",
            funnel.follow_up_collaborators.to_string(),
        ),
    ] {
        writer.write_record(["funnel", label, "", &value])?;
    }
    for row in &growth.source_breakdown {
        let label = spreadsheet_safe_text(Some(&row.label));
        writer.write_record(["source", &label, "", &row.total.to_string()])?;
    }
    for row in &growth.referral_breakdown {
        let label = spreadsheet_safe_text(Some(&row.label));
        writer.write_record(["referral", &label, "", &row.total.to_string()])?;
    }
    for totals in &growth.finances {
        for (label, value) in [
            ("gross_purchases_minor", totals.gross_purchases_minor),
            ("refunds_minor", totals.refunds_minor),
            ("net_purchases_minor", totals.net_purchases_minor),
            ("manual_income_minor", totals.manual_income_minor),
            ("manual_expense_minor", totals.manual_expense_minor),
            ("net_total_minor", totals.net_total_minor),
        ] {
            writer.write_record(["finance", label, &totals.currency_code, &value.to_string()])?;
        }
    }
    for entry in &growth.finance_entries {
        let label = spreadsheet_safe_text(Some(&format!("{}/{}", entry.kind, entry.category)));
        writer.write_record([
            "manual_entry",
            &label,
            &entry.currency_code,
            &entry.amount_minor.to_string(),
        ])?;
    }
    if let Some(report) = &growth.sponsor_report {
        for sponsor in &report.sponsors {
            for (label, value) in [
                ("impressions", sponsor.impressions),
                ("clicks", sponsor.clicks),
                ("leads", i64::from(sponsor.leads_count)),
                ("conversations", i64::from(sponsor.conversations_count)),
                ("meetings", i64::from(sponsor.meetings_count)),
                ("promised_deliverables", sponsor.promised_deliverables),
                ("delivered_deliverables", sponsor.delivered_deliverables),
            ] {
                let label = spreadsheet_safe_text(Some(&format!("{}/{}", sponsor.name, label)));
                writer.write_record(["sponsor", &label, "", &value.to_string()])?;
            }
        }
    }
    let bytes = writer.into_inner().map_err(csv::IntoInnerError::into_error)?;
    Ok(String::from_utf8(bytes)?)
}

/// Builds the database payload for an event form.
fn build_event_payload(event: &Event) -> Result<serde_json::Value, HandlerError> {
    event
        .to_db_payload()
        .map_err(|err| HandlerError::Deserialization(err.to_string()))
}

/// Removes volatile event fields before storing an event as reusable group defaults.
fn sanitize_event_defaults(event: Value) -> Option<Value> {
    let Value::Object(mut payload) = event else {
        return None;
    };

    for key in [
        "attendee_count",
        "canceled",
        "created_at",
        "ends_at",
        "event_id",
        "event_series_id",
        "has_related_events",
        "meeting_error",
        "meeting_in_sync",
        "meeting_join_url",
        "meeting_password",
        "meeting_recording_raw_urls",
        "meeting_recording_url",
        "published",
        "published_at",
        "registration_ends_at",
        "registration_starts_at",
        "sessions",
        "slug",
        "starts_at",
        "waitlist_count",
    ] {
        payload.remove(key);
    }

    clear_nested_ids(&mut payload);

    Some(Value::Object(payload))
}

/// Clears IDs from nested collections so defaults create fresh child rows.
fn clear_nested_ids(payload: &mut Map<String, Value>) {
    clear_array_object_ids(payload, "discount_codes", &["event_discount_code_id"]);
    clear_array_object_ids(payload, "ticket_types", &["event_ticket_type_id"]);
    if let Some(Value::Array(ticket_types)) = payload.get_mut("ticket_types") {
        for ticket_type in ticket_types {
            if let Value::Object(ticket_type) = ticket_type {
                clear_array_object_ids(
                    ticket_type,
                    "price_windows",
                    &["event_ticket_price_window_id"],
                );
            }
        }
    }
    clear_array_object_ids(payload, "cfs_labels", &["event_cfs_label_id"]);
}

/// Removes identifier keys from each object in an array field.
fn clear_array_object_ids(payload: &mut Map<String, Value>, field: &str, keys: &[&str]) {
    let Some(Value::Array(items)) = payload.get_mut(field) else {
        return;
    };

    for item in items {
        if let Value::Object(item) = item {
            for key in keys {
                item.remove(*key);
            }
        }
    }
}

/// Builds a `HashMap` of meeting provider to max participants from config.
fn build_meetings_max_participants(
    meetings_cfg: Option<&MeetingsConfig>,
) -> HashMap<MeetingProvider, i32> {
    let mut map = HashMap::new();
    if let Some(cfg) = meetings_cfg
        && let Some(google_meet) = &cfg.google_meet
    {
        map.insert(MeetingProvider::GoogleMeet, google_meet.max_participants);
    }
    if let Some(cfg) = meetings_cfg
        && let Some(zoom) = &cfg.zoom
    {
        map.insert(MeetingProvider::Zoom, zoom.max_participants);
    }
    map
}

/// Ensures that ticketing can be used for the event by checking payments configuration and group setup.
async fn ensure_ticketing_ready(
    db: &DynDB,
    alliance_id: Uuid,
    group_id: Uuid,
    payments_cfg: Option<&PaymentsConfig>,
) -> Result<(), HandlerError> {
    // Require a configured server payments provider before enabling ticketing
    let Some(payments_cfg) = payments_cfg else {
        return Err(HandlerError::Database(
            "payments are not configured on this server".to_string(),
        ));
    };

    // Require a group recipient that matches the configured payments provider
    let payment_recipient = db.get_group_payment_recipient(alliance_id, group_id).await?;
    if payment_recipient.is_none() {
        return Err(HandlerError::Database(
            "configure a payments recipient in group settings first".to_string(),
        ));
    }

    if !payments_ready(payment_recipient.as_ref(), Some(payments_cfg)) {
        return Err(HandlerError::Database(
            "configure a payments recipient for this server's payments provider first".to_string(),
        ));
    }

    Ok(())
}

/// Resolves the event identifiers affected by a dashboard event action.
async fn event_action_ids(
    db: &dyn DBOperations,
    group_id: Uuid,
    event_id: Uuid,
    scope: EventActionScope,
) -> Result<Vec<Uuid>> {
    if scope == EventActionScope::This {
        return Ok(vec![event_id]);
    }

    let event_ids = db.list_event_series_event_ids(group_id, event_id).await?;
    if event_ids.is_empty() {
        Ok(vec![event_id])
    } else {
        Ok(event_ids)
    }
}

/// Checks if the event payload includes ticket types, indicating that ticketing is used.
fn event_payload_uses_ticketing(event_payload: &serde_json::Value) -> bool {
    event_payload
        .get("ticket_types")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|ticket_types| !ticket_types.is_empty())
}

/// Parses dashboard event action query parameters.
fn parse_event_action_query(raw_query: Option<&str>) -> Result<EventActionQuery, HandlerError> {
    Ok(serde_qs_config().deserialize_str(raw_query.unwrap_or_default())?)
}

/// Checks whether group payments are ready for ticketed events.
fn payments_ready(
    payment_recipient: Option<&GroupPaymentRecipient>,
    payments_cfg: Option<&PaymentsConfig>,
) -> bool {
    matches!(
        (payment_recipient, payments_cfg),
        (Some(payment_recipient), Some(payments_cfg))
            if payment_recipient.provider == payments_cfg.provider()
    )
}

/// Prepares the events list page and filters for the group dashboard.
pub(crate) async fn prepare_list_page(
    db: &DynDB,
    alliance_id: Uuid,
    group_id: Uuid,
    user_id: Uuid,
    raw_query: &str,
) -> Result<(EventsListFilters, events::ListPage), HandlerError> {
    // Fetch group's past and upcoming events
    let filters: EventsListFilters = serde_qs_config().deserialize_str(raw_query)?;
    let (can_manage_events, events) = tokio::try_join!(
        db.user_has_group_permission(
            &alliance_id,
            &group_id,
            &user_id,
            GroupPermission::EventsWrite
        ),
        db.list_group_events(group_id, &filters)
    )?;

    // Prepare pagination links for each events tab
    let mut past_filters = filters.clone();
    past_filters.events_tab = Some(EventsTab::Past);
    let mut upcoming_filters = filters.clone();
    upcoming_filters.events_tab = Some(EventsTab::Upcoming);
    let past_navigation_links = NavigationLinks::from_filters(
        &past_filters,
        events.past.total,
        DASHBOARD_URL,
        PARTIAL_URL,
    )?;
    let upcoming_navigation_links = NavigationLinks::from_filters(
        &upcoming_filters,
        events.upcoming.total,
        DASHBOARD_URL,
        PARTIAL_URL,
    )?;

    // Prepare template
    let template = events::ListPage {
        can_manage_events,
        events,
        events_tab: filters.current_tab(),
        past_navigation_links,
        upcoming_navigation_links,
        limit: filters.limit,
        past_offset: filters.past_offset,
        upcoming_offset: filters.upcoming_offset,
    };

    Ok((filters, template))
}
