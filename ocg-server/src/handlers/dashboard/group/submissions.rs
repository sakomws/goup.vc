//! HTTP handlers for event CFS submissions in the group dashboard.

use askama::Template;
use axum::{
    extract::{Path, RawQuery, State},
    http::{
        HeaderMap, StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    },
    response::{Html, IntoResponse},
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use tracing::{instrument, warn};
use uuid::Uuid;

use crate::{
    config::HttpServerConfig,
    db::{DynDB, notifications::CustomNotificationTracking},
    handlers::{
        error::HandlerError,
        extractors::{CurrentUser, SelectedAllianceId, SelectedGroupId, ValidatedFormQs},
    },
    router::serde_qs_config,
    services::notifications::{DynNotificationsManager, NewNotification, NotificationKind},
    templates::{
        dashboard::group::submissions::{
            self, CfsLabelFilterOption, CfsSubmission, CfsSubmissionPrepEmailContext,
            CfsSubmissionUpdate, CfsSubmissionsFilters,
        },
        notifications::{CfsSubmissionUpdated, EventCustom},
    },
    types::{
        pagination::{self, NavigationLinks},
        permissions::GroupPermission,
    },
    validation::{MAX_LEN_M, MAX_LEN_NOTIFICATION_BODY, MAX_PAGINATION_LIMIT, trimmed_non_empty},
};

#[cfg(test)]
mod tests;

// Pages handlers.

/// Displays the CFS submissions list for an event.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    // Fetch event submissions (checking event belongs to group)
    let filters: CfsSubmissionsFilters =
        serde_qs_config().deserialize_str(raw_query.as_deref().unwrap_or_default())?;
    let (can_manage_events, event, labels, statuses, submissions) = tokio::try_join!(
        db.user_has_group_permission(
            &alliance_id,
            &group_id,
            &user.user_id,
            GroupPermission::EventsWrite
        ),
        db.get_event_summary(alliance_id, group_id, event_id), // ensure event belongs to group
        db.list_event_cfs_labels(event_id),
        db.list_cfs_submission_statuses_for_review(),
        db.list_event_cfs_submissions(event_id, &filters)
    )?;

    // Prepare template
    let base_path = format!("/dashboard/group/events/{event_id}/submissions");
    let navigation_links =
        NavigationLinks::from_filters(&filters, submissions.total, &base_path, &base_path)?;
    let refresh_url = pagination::build_url(&base_path, &filters)?;
    let event_cfs_labels = labels
        .into_iter()
        .map(|label| {
            let submissions_count = label
                .event_cfs_label_id
                .and_then(|label_id| {
                    submissions
                        .label_counts
                        .iter()
                        .find(|count| count.event_cfs_label_id == label_id)
                })
                .map_or(0, |count| count.submissions_count);
            CfsLabelFilterOption {
                color: label.color,
                event_cfs_label_id: label.event_cfs_label_id,
                name: label.name,
                submissions_count,
            }
        })
        .collect();
    let template = submissions::ListPage {
        can_manage_events,
        event_canceled: event.canceled,
        event_has_venue: event
            .venue_name
            .as_deref()
            .is_some_and(|name| !name.trim().is_empty()),
        event_cfs_labels,
        event_id,
        statuses,
        submissions: submissions.submissions,
        navigation_links,
        no_labels: filters.no_labels == Some(true),
        refresh_url,
        selected_event_cfs_label_ids: filters.label_ids.clone(),
        sort: filters.sort.clone().unwrap_or_else(|| "created-desc".to_string()),
        ts_query: filters.ts_query.clone(),
        total: submissions.total,
        unlabeled_submissions_count: submissions.unlabeled_count,
        limit: filters.limit,
        offset: filters.offset,
    };

    Ok(Html(template.render()?))
}

/// Downloads all event submissions with speaker and proposal details.
#[instrument(skip_all, err)]
pub(crate) async fn download_csv(
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    Path(event_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let event = db.get_event_summary(alliance_id, group_id, event_id).await?;
    let mut submissions = Vec::new();
    let mut offset = 0;

    loop {
        let filters = CfsSubmissionsFilters {
            label_ids: None,
            no_labels: None,
            limit: Some(MAX_PAGINATION_LIMIT),
            offset: Some(offset),
            sort: Some("created-asc".to_string()),
            ts_query: None,
        };
        let output = db.list_event_cfs_submissions(event_id, &filters).await?;
        let page_len = output.submissions.len();
        submissions.extend(output.submissions);
        offset += page_len;
        if page_len == 0 || offset >= output.total {
            break;
        }
    }

    let csv = build_submissions_csv(&submissions)?;
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "text/csv; charset=utf-8".parse().unwrap());
    headers.insert(
        CONTENT_DISPOSITION,
        format!(
            "attachment; filename=\"event-{}-submissions.csv\"",
            event.slug
        )
        .parse()
        .unwrap(),
    );

    Ok((headers, csv))
}

// Action handlers.

/// Updates a CFS submission for an event.
#[allow(clippy::too_many_arguments)]
#[instrument(skip_all, err)]
pub(crate) async fn update(
    CurrentUser(reviewer): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    State(notifications_manager): State<DynNotificationsManager>,
    State(server_cfg): State<HttpServerConfig>,
    Path((event_id, cfs_submission_id)): Path<(Uuid, Uuid)>,
    ValidatedFormQs(update): ValidatedFormQs<CfsSubmissionUpdate>,
) -> Result<impl IntoResponse, HandlerError> {
    // Ensure event belongs to the group
    let event = db.get_event_summary(alliance_id, group_id, event_id).await?;

    // Update submission in database
    let should_notify = db
        .update_cfs_submission(reviewer.user_id, event_id, cfs_submission_id, &update)
        .await?;

    if should_notify {
        // Enqueue notification to submission author best-effort
        if let Err(err) = async {
            let (notification_data, site_settings) = tokio::try_join!(
                db.get_cfs_submission_notification_data(event_id, cfs_submission_id),
                db.get_site_settings(),
            )?;
            let base_url = server_cfg.base_url.strip_suffix('/').unwrap_or(&server_cfg.base_url);
            let link = format!("{base_url}/dashboard/user?tab=submissions");
            let template_data = CfsSubmissionUpdated {
                action_required_message: notification_data.action_required_message,
                event,
                link,
                status_name: notification_data.status_name,
                theme: site_settings.theme,
            };
            let notification = NewNotification {
                attachments: vec![],
                kind: NotificationKind::CfsSubmissionUpdated,
                recipients: vec![notification_data.user_id],
                template_data: Some(serde_json::to_value(&template_data)?),
            };
            notifications_manager.enqueue(&notification).await
        }
        .await
        {
            warn!(
                error = %err,
                %event_id,
                %cfs_submission_id,
                reviewer_id = %reviewer.user_id,
                "failed to enqueue CFS submission update notification"
            );
        }
    }

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-event-submissions")],
    ))
}

fn csv_safe(value: &str) -> String {
    let trimmed = value.trim_start();
    if trimmed.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{value}")
    } else {
        value.to_string()
    }
}

fn build_submissions_csv(submissions: &[CfsSubmission]) -> anyhow::Result<Vec<u8>> {
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::Any(b'\n'))
        .from_writer(Vec::new());
    writer.write_record([
        "submission_id",
        "status",
        "track",
        "talk_title",
        "talk_description",
        "duration_minutes",
        "level",
        "speaker_name",
        "speaker_username",
        "speaker_title",
        "speaker_company",
        "co_speaker_name",
        "co_speaker_username",
        "average_rating",
        "ratings_count",
        "submitted_at",
    ])?;

    for submission in submissions {
        let speaker = &submission.speaker;
        let co_speaker = submission.session_proposal.co_speaker.as_ref();
        let tracks = submission
            .labels
            .iter()
            .map(|label| label.name.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        writer.write_record([
            submission.cfs_submission_id.to_string(),
            csv_safe(&submission.status_name),
            csv_safe(&tracks),
            csv_safe(&submission.session_proposal.title),
            csv_safe(submission.session_proposal.description.as_deref().unwrap_or_default()),
            submission
                .session_proposal
                .duration_minutes
                .map(|value| value.to_string())
                .unwrap_or_default(),
            csv_safe(
                submission
                    .session_proposal
                    .session_proposal_level_name
                    .as_deref()
                    .unwrap_or_default(),
            ),
            csv_safe(speaker.name.as_deref().unwrap_or(&speaker.username)),
            csv_safe(&speaker.username),
            csv_safe(speaker.title.as_deref().unwrap_or_default()),
            csv_safe(speaker.company.as_deref().unwrap_or_default()),
            csv_safe(
                co_speaker
                    .and_then(|speaker| speaker.name.as_deref())
                    .unwrap_or_default(),
            ),
            csv_safe(
                co_speaker
                    .map(|speaker| speaker.username.as_str())
                    .unwrap_or_default(),
            ),
            submission
                .average_rating
                .map(|value| format!("{value:.2}"))
                .unwrap_or_default(),
            submission.ratings_count.to_string(),
            submission.created_at.to_rfc3339(),
        ])?;
    }

    Ok(writer.into_inner().map_err(csv::IntoInnerError::into_error)?)
}

/// Emails speakers for approved, rejected, or all active submissions.
#[instrument(skip_all, err)]
pub(crate) async fn send_email(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    State(server_cfg): State<HttpServerConfig>,
    Path(event_id): Path<Uuid>,
    ValidatedFormQs(email): ValidatedFormQs<CfsSubmissionEmail>,
) -> Result<impl IntoResponse, HandlerError> {
    let (site_settings, event, recipients) = tokio::try_join!(
        db.get_site_settings(),
        db.get_event_summary(alliance_id, group_id, event_id),
        db.resolve_event_cfs_email_recipient_ids(group_id, event_id, email.audience.as_ref()),
    )?;

    if event.canceled {
        return Ok((
            StatusCode::BAD_REQUEST,
            "Canceled events cannot send speaker emails.",
        )
            .into_response());
    }

    if recipients.is_empty() {
        let message = match email.audience {
            CfsEmailAudience::Approved => {
                "No approved speakers with verified email addresses who accept messages from event organizers."
            }
            CfsEmailAudience::Rejected => {
                "No rejected speakers with verified email addresses who accept messages from event organizers."
            }
            CfsEmailAudience::All => {
                "No speakers with verified email addresses who accept messages from event organizers."
            }
        };
        return Ok((StatusCode::BAD_REQUEST, message).into_response());
    }

    let base_url = server_cfg.base_url.strip_suffix('/').unwrap_or(&server_cfg.base_url);
    let link = format!(
        "{}/{}/group/{}/event/{}",
        base_url,
        event.alliance_name,
        event.public_group_slug(),
        event.slug
    );
    let template_data = EventCustom {
        body: email.body.clone(),
        event,
        link,
        subject: email.subject.clone(),
        theme: site_settings.theme,
    };
    let notification = NewNotification {
        attachments: vec![],
        kind: NotificationKind::EventCustom,
        recipients,
        template_data: Some(serde_json::to_value(&template_data)?),
    };
    db.enqueue_tracked_custom_notification(
        &notification,
        CustomNotificationTracking {
            alliance_id: Some(alliance_id),
            body: email.body.clone(),
            created_by: user.user_id,
            event_id: Some(event_id),
            group_id: Some(group_id),
            recipient_count: notification.recipients.len(),
            subject: email.subject.clone(),
        },
    )
    .await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Emails one speaker prep step and records that it was sent.
#[instrument(skip_all, err)]
pub(crate) async fn send_prep_email(
    CurrentUser(user): CurrentUser,
    SelectedAllianceId(alliance_id): SelectedAllianceId,
    SelectedGroupId(group_id): SelectedGroupId,
    State(db): State<DynDB>,
    State(server_cfg): State<HttpServerConfig>,
    Path((event_id, cfs_submission_id)): Path<(Uuid, Uuid)>,
    ValidatedFormQs(email): ValidatedFormQs<CfsPrepEmail>,
) -> Result<impl IntoResponse, HandlerError> {
    let (site_settings, event, context) = tokio::try_join!(
        db.get_site_settings(),
        db.get_event_summary(alliance_id, group_id, event_id),
        db.get_cfs_submission_prep_email_context(group_id, event_id, cfs_submission_id),
    )?;
    let Some(context) = context else {
        return Err(HandlerError::NotFound);
    };

    if let Some(message) = prep_email_block(&event, &context, email.step) {
        return Ok((StatusCode::BAD_REQUEST, message).into_response());
    }

    let base_url = server_cfg.base_url.strip_suffix('/').unwrap_or(&server_cfg.base_url);
    let link = format!(
        "{}/{}/group/{}/event/{}",
        base_url,
        event.alliance_name,
        event.public_group_slug(),
        event.slug
    );
    let (subject, body) = prep_email_content(email.step, &event, &context, base_url);
    let template_data = EventCustom {
        body: body.clone(),
        event,
        link,
        subject: subject.clone(),
        theme: site_settings.theme,
    };
    db.send_cfs_submission_prep_email(
        user.user_id,
        group_id,
        event_id,
        cfs_submission_id,
        email.step.as_ref(),
        &serde_json::to_value(&template_data)?,
        &subject,
        &body,
    )
    .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-event-submissions")],
    )
        .into_response())
}

fn prep_email_block(
    event: &crate::types::event::EventSummary,
    context: &CfsSubmissionPrepEmailContext,
    step: PrepEmailStep,
) -> Option<&'static str> {
    if event.canceled {
        return Some("Canceled events cannot send speaker emails.");
    }
    if context.status_id != "approved" {
        return Some("Only approved submissions can receive speaker prep emails.");
    }
    if context.prep_email_sent(step.as_ref()) {
        return Some("This speaker prep email was already sent.");
    }
    match step {
        PrepEmailStep::Listed if !context.has_session() => {
            Some("Add this talk to the agenda before emailing that it is listed.")
        }
        PrepEmailStep::Schedule if context.tracks.is_empty() => {
            Some("Assign a track before emailing the schedule.")
        }
        PrepEmailStep::Schedule if context.session_starts_at.is_none() => {
            Some("Set the session time before emailing the schedule.")
        }
        PrepEmailStep::Schedule
            if context
                .venue_name
                .as_deref()
                .is_none_or(|name| name.trim().is_empty()) =>
        {
            Some("Add the event venue before emailing the schedule.")
        }
        PrepEmailStep::Listed
        | PrepEmailStep::Profile
        | PrepEmailStep::Slides
        | PrepEmailStep::Schedule => None,
    }
}

fn prep_email_content(
    step: PrepEmailStep,
    event: &crate::types::event::EventSummary,
    context: &CfsSubmissionPrepEmailContext,
    base_url: &str,
) -> (String, String) {
    let title = &context.title;
    match step {
        PrepEmailStep::Listed => (
            format!("You're on the speakers list for {}", event.name),
            format!("Your talk \"{title}\" is on the speakers list."),
        ),
        PrepEmailStep::Profile => (
            format!("Add your photo and bio for {}", event.name),
            format!(
                "Please add a photo and a short bio on your account so we can use them on the speaker poster.\n\n{base_url}/dashboard/user?tab=account"
            ),
        ),
        PrepEmailStep::Slides => (
            format!("Add your slides for {}", event.name),
            format!(
                "Please add your slides for \"{title}\" to the shared speaker deck and reply to this email with the link."
            ),
        ),
        PrepEmailStep::Schedule => {
            let starts_at = context
                .session_starts_at
                .expect("schedule email requires a session time");
            let when = format!(
                "{} {}",
                starts_at.with_timezone(&event.timezone).format("%b %d, %Y %H:%M"),
                event.timezone
            );
            let tracks = context.tracks.join(", ");
            let venue = context.venue_name.clone().unwrap_or_default();
            (
                format!("Your time slot for {}", event.name),
                format!(
                    "Your talk \"{title}\" is on the {tracks} track at {when}. The venue is {venue}. The event page has the schedule."
                ),
            )
        }
    }
}

/// Form data for emailing one speaker prep step.
#[derive(Debug, Deserialize, Serialize, Validate)]
pub(crate) struct CfsPrepEmail {
    /// Speaker prep step to email.
    #[garde(skip)]
    pub step: PrepEmailStep,
}

/// Speaker prep email tracked on an approved submission.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Serialize, strum::AsRefStr)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PrepEmailStep {
    /// The talk is on the public speakers list.
    #[strum(serialize = "listed")]
    Listed,
    /// The speaker should add a photo and bio.
    #[strum(serialize = "profile")]
    Profile,
    /// The speaker should add slides.
    #[strum(serialize = "slides")]
    Slides,
    /// Track, time, and venue are ready.
    #[strum(serialize = "schedule")]
    Schedule,
}

/// Form data for emailing submission speakers.
#[derive(Debug, Deserialize, Serialize, Validate)]
pub(crate) struct CfsSubmissionEmail {
    /// Which submissions' speakers should receive the email.
    #[garde(skip)]
    pub audience: CfsEmailAudience,
    /// Body text for the notification.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_NOTIFICATION_BODY))]
    pub body: String,
    /// Subject line for the notification email.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub subject: String,
}

/// Speaker audience for a submission email.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Serialize, strum::AsRefStr)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CfsEmailAudience {
    /// Speakers and co-speakers on every submission except withdrawn ones.
    #[strum(serialize = "all")]
    All,
    /// Speakers and co-speakers on approved submissions.
    #[strum(serialize = "approved")]
    Approved,
    /// Speakers and co-speakers on rejected submissions.
    #[strum(serialize = "rejected")]
    Rejected,
}
