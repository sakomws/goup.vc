//! HTTP handlers for mock interview practice pages.

use askama::Template;
use axum::{
    extract::{Path, RawQuery, State},
    http::{StatusCode, header::CACHE_CONTROL},
    response::{Html, IntoResponse, Redirect},
};
use axum_messages::Messages;
use garde::Validate;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{CurrentUser, ValidatedForm},
    },
    router::{CACHE_CONTROL_PRIVATE_NO_STORE, serde_qs_config},
    templates::{PageId, auth::User, site::mock_interviews},
    types::mock_interviews::{
        INTERVIEW_TYPES, MockInterviewIntervieweeFeedbackInput,
        MockInterviewInterviewerFeedbackInput, MockInterviewMatchFilters, MockInterviewProfileInput,
        MockInterviewRequestInput, MockInterviewRespondInput, TARGET_COMPANY_TYPES,
    },
};

const BASE_URL: &str = "/mock-interviews";

/// Render the mock interviews landing page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    let site_settings = db.get_site_settings().await?;
    let user = User::from_session(auth_session.clone()).await?;
    let profile = if let Some(session_user) = auth_session.user.as_ref() {
        db.get_mock_interview_profile(session_user.user_id).await?
    } else {
        None
    };

    let template = mock_interviews::Page {
        page_id: PageId::SiteMockInterviews,
        path: BASE_URL.to_string(),
        site_settings,
        user,
        profile,
    };

    Ok((
        [(CACHE_CONTROL, CACHE_CONTROL_PRIVATE_NO_STORE)],
        Html(template.render()?),
    ))
}

/// Render onboarding/profile setup.
#[instrument(skip_all, err)]
pub(crate) async fn onboarding_page(
    auth_session: AuthSession,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    let site_settings = db.get_site_settings().await?;
    let auth_user = User::from_session(auth_session).await?;
    let profile = db.get_mock_interview_profile(user.user_id).await?;

    let template = mock_interviews::OnboardingPage {
        page_id: PageId::SiteMockInterviews,
        path: format!("{BASE_URL}/onboarding"),
        site_settings,
        user: auth_user,
        profile,
        interview_types: INTERVIEW_TYPES,
        target_company_types: TARGET_COMPANY_TYPES,
    };

    Ok((
        [(CACHE_CONTROL, CACHE_CONTROL_PRIVATE_NO_STORE)],
        Html(template.render()?),
    ))
}

/// Save onboarding profile.
#[instrument(skip_all, err)]
pub(crate) async fn save_onboarding(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<MockInterviewProfileInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.upsert_mock_interview_profile(user.user_id, &input).await?;
    messages.success("Mock interview profile saved.");
    Ok(Redirect::to(&format!("{BASE_URL}/matches")))
}

/// Render suggested matches.
#[instrument(skip_all, err)]
pub(crate) async fn matches_page(
    auth_session: AuthSession,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    let site_settings = db.get_site_settings().await?;
    let auth_user = User::from_session(auth_session).await?;

    let filters = parse_match_filters(raw_query.as_deref().unwrap_or_default())?;
    let profile = db.get_mock_interview_profile(user.user_id).await?;
    let output = db
        .search_mock_interview_matches(user.user_id, &filters)
        .await?;

    let template = mock_interviews::MatchesPage {
        page_id: PageId::SiteMockInterviews,
        path: format!("{BASE_URL}/matches"),
        site_settings,
        user: auth_user,
        profile,
        filters,
        matches: output.matches,
        total: output.total,
        interview_types: INTERVIEW_TYPES,
    };

    Ok((
        [(CACHE_CONTROL, CACHE_CONTROL_PRIVATE_NO_STORE)],
        Html(template.render()?),
    ))
}

/// Render requests dashboard.
#[instrument(skip_all, err)]
pub(crate) async fn requests_page(
    auth_session: AuthSession,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    let site_settings = db.get_site_settings().await?;
    let auth_user = User::from_session(auth_session).await?;
    let requests = db.list_mock_interview_requests(user.user_id).await?;

    let template = mock_interviews::RequestsPage {
        page_id: PageId::SiteMockInterviews,
        path: format!("{BASE_URL}/requests"),
        site_settings,
        user: auth_user,
        current_user_id: user.user_id,
        requests,
    };

    Ok((
        [(CACHE_CONTROL, CACHE_CONTROL_PRIVATE_NO_STORE)],
        Html(template.render()?),
    ))
}

/// Create a mock interview request.
#[instrument(skip_all, err)]
pub(crate) async fn create_request(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    ValidatedForm(input): ValidatedForm<MockInterviewRequestInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.add_mock_interview_request(user.user_id, &input).await?;
    messages.success("Mock interview request sent.");
    Ok(Redirect::to(&format!("{BASE_URL}/requests")))
}

/// Accept, decline, or cancel a request.
#[instrument(skip_all, err)]
pub(crate) async fn respond_request(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(request_id): Path<Uuid>,
    ValidatedForm(input): ValidatedForm<MockInterviewRespondInput>,
) -> Result<impl IntoResponse, HandlerError> {
    let session_id = db
        .respond_mock_interview_request(
            user.user_id,
            request_id,
            &input.action,
            input.meeting_url.as_deref(),
        )
        .await?;

    match input.action.as_str() {
        "accept" => {
            messages.success("Mock interview accepted.");
            if let Some(session_id) = session_id {
                return Ok(Redirect::to(&format!("{BASE_URL}/session/{session_id}")));
            }
        }
        "decline" => messages.success("Request declined."),
        "cancel" => messages.success("Request cancelled."),
        _ => {}
    }

    Ok(Redirect::to(&format!("{BASE_URL}/requests")))
}

/// Render a session page.
#[instrument(skip_all, err)]
pub(crate) async fn session_page(
    auth_session: AuthSession,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(session_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let site_settings = db.get_site_settings().await?;
    let auth_user = User::from_session(auth_session).await?;
    let session = db
        .get_mock_interview_session(user.user_id, session_id)
        .await?
        .ok_or(HandlerError::NotFound)?;

    let template = mock_interviews::SessionPage {
        page_id: PageId::SiteMockInterviews,
        path: format!("{BASE_URL}/session/{session_id}"),
        site_settings,
        user: auth_user,
        current_user_id: user.user_id,
        session,
    };

    Ok((
        [(CACHE_CONTROL, CACHE_CONTROL_PRIVATE_NO_STORE)],
        Html(template.render()?),
    ))
}

/// Submit session feedback.
#[instrument(skip_all, err)]
pub(crate) async fn submit_feedback(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(session_id): Path<Uuid>,
    ValidatedForm(input): ValidatedForm<MockInterviewInterviewerFeedbackInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.add_mock_interview_interviewer_feedback(user.user_id, session_id, &input)
        .await?;
    messages.success("Feedback submitted.");
    Ok(Redirect::to(&format!("{BASE_URL}/session/{session_id}")))
}

/// Submit interviewee feedback.
#[instrument(skip_all, err)]
pub(crate) async fn submit_interviewee_feedback(
    messages: Messages,
    CurrentUser(user): CurrentUser,
    State(db): State<DynDB>,
    Path(session_id): Path<Uuid>,
    ValidatedForm(input): ValidatedForm<MockInterviewIntervieweeFeedbackInput>,
) -> Result<impl IntoResponse, HandlerError> {
    db.add_mock_interview_interviewee_feedback(user.user_id, session_id, &input)
        .await?;
    messages.success("Feedback submitted.");
    Ok(Redirect::to(&format!("{BASE_URL}/session/{session_id}")))
}

fn parse_match_filters(raw_query: &str) -> Result<MockInterviewMatchFilters, HandlerError> {
    let filters: MockInterviewMatchFilters = if raw_query.is_empty() {
        MockInterviewMatchFilters::default()
    } else {
        serde_qs_config().deserialize_str(raw_query)?
    };
    filters.validate()?;
    Ok(filters)
}
