//! HTTP routing configuration for the OCG server.
//!
//! This module sets up the Axum router with all application routes, middleware layers,
//! and static file handling.

mod api;
mod dashboard;

#[cfg(test)]
mod tests;

use std::{
    collections::HashMap,
    sync::LazyLock,
    time::{Duration, Instant},
};

use anyhow::Result;
use axum::{
    Router,
    extract::{FromRef, Request, State as AxumState},
    http::{
        HeaderName, HeaderValue, Method, StatusCode, Uri,
        header::{CACHE_CONTROL, CONTENT_TYPE, HOST, VARY},
    },
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, post, put},
};
use axum_login::login_required;
use axum_messages::MessagesManagerLayer;
use rust_embed::Embed;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{set_header::SetResponseHeaderLayer, trace::TraceLayer};
use tracing::instrument;

use crate::{
    activity_tracker::DynActivityTracker,
    auth::AuthnBackend,
    config::{HttpServerConfig, MeetingsConfig, PaymentsConfig, YouComConfig},
    db::{DynDB, PgDB},
    handlers::{
        alliance,
        auth::{self, LOG_IN_URL},
        event, group, images, meetings, payments, site,
    },
    services::{
        event_discovery::ManualEventDiscovery, images::DynImageStorage,
        job_discovery::ManualJobDiscovery, notifications::DynNotificationsManager,
        payments::DynPaymentsManager,
    },
    types::custom_domain::CustomDomainTarget,
};

pub(crate) const VERIFIED_CUSTOM_DOMAIN_HEADER: &str = "x-ocg-verified-custom-domain";
const CUSTOM_DOMAIN_CACHE_TTL: Duration = Duration::from_mins(1);
const CUSTOM_DOMAIN_NEGATIVE_CACHE_TTL: Duration = Duration::from_secs(5);
const CUSTOM_DOMAIN_CACHE_MAX_ENTRIES: usize = 1024;

#[derive(Clone)]
struct CachedCustomDomain {
    expires_at: Instant,
    target: Option<CustomDomainTarget>,
}

static CUSTOM_DOMAIN_CACHE: LazyLock<RwLock<HashMap<String, CachedCustomDomain>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Cache-Control header value for immutable public assets.
#[cfg(any(not(debug_assertions), test))]
pub(crate) const CACHE_CONTROL_IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// Cache-Control header value instructing clients and proxies not to store responses.
pub(crate) const CACHE_CONTROL_NO_STORE: &str = "no-store";

/// Cache-Control header value for private responses that must not be stored.
pub(crate) const CACHE_CONTROL_PRIVATE_NO_STORE: &str = "private, no-store";

/// Cache-Control header value for shared public responses.
#[cfg(any(not(debug_assertions), test))]
pub(crate) const CACHE_CONTROL_PUBLIC_SHARED: &str =
    "public, max-age=0, s-maxage=300, stale-while-revalidate=60";

/// Cache-Control header value for shared public responses in local development.
#[cfg(all(debug_assertions, not(test)))]
pub(crate) const CACHE_CONTROL_PUBLIC_SHARED: &str = "public, max-age=0";

/// Cache-Control header value for favicon redirects.
const CACHE_CONTROL_FAVICON_REDIRECT: &str = "no-cache";

/// Cache-Control header value for default static asset responses.
#[cfg(any(not(debug_assertions), test))]
const CACHE_CONTROL_STATIC_DEFAULT: &str = "max-age=3600";

/// Cache-Control header value for local development static asset responses.
#[cfg(all(debug_assertions, not(test)))]
const CACHE_CONTROL_STATIC_DEVELOPMENT: &str = "max-age=0";

/// Cache-Control header value for static image responses.
#[cfg(any(not(debug_assertions), test))]
const CACHE_CONTROL_STATIC_IMAGES: &str = "max-age=604800";

/// Current application commit SHA embedded at build time.
pub(crate) const COMMIT_SHA: &str = env!("OCG_COMMIT_SHA");

/// Header carrying the loaded or current application commit SHA.
pub(crate) const COMMIT_SHA_HEADER: &str = "x-ocg-commit-sha";

/// Header used to request an application-level page refresh for `ocgFetch`.
const OCG_REFRESH_HEADER: &str = "x-ocg-refresh";

/// Headers for public shared-cache responses without additional headers.
pub(crate) const PUBLIC_SHARED_CACHE_HEADERS: [(HeaderName, &str); 2] = [
    (CACHE_CONTROL, CACHE_CONTROL_PUBLIC_SHARED),
    (VARY, PUBLIC_SHARED_CACHE_VARY),
];

/// Vary header value for public shared-cache responses.
pub(crate) const PUBLIC_SHARED_CACHE_VARY: &str = "x-ocg-commit-sha, hx-request, x-ocg-fetch";

/// Static file embedder using rust-embed.
///
/// Embeds all files from the static directory into the binary.
#[derive(Embed)]
#[folder = "dist/static"]
struct StaticFile;

/// Shared state for the router.
#[derive(Clone, FromRef)]
pub(crate) struct State {
    /// Activity tracker handle.
    pub activity_tracker: DynActivityTracker,
    /// Database handle.
    pub db: DynDB,
    /// Image storage provider handle.
    pub image_storage: DynImageStorage,
    /// Optional You.com discovery runner for authorized dashboard actions.
    pub manual_event_discovery: Option<ManualEventDiscovery>,
    /// Optional You.com discovery runner for global jobs dashboard actions.
    pub manual_job_discovery: Option<ManualJobDiscovery>,
    /// Meetings configuration.
    pub meetings_cfg: Option<MeetingsConfig>,
    /// Notifications manager handle.
    pub notifications_manager: DynNotificationsManager,
    /// Payments configuration.
    pub payments_cfg: Option<PaymentsConfig>,
    /// Payments manager handle.
    pub payments_manager: DynPaymentsManager,
    /// `serde_qs` config for query string parsing.
    pub serde_qs_de: serde_qs::Config,
    /// HTTP server configuration.
    pub server_cfg: HttpServerConfig,
}

/// Configures and returns the application router.
///
/// Sets up all routes, middleware layers, and shared state. Optionally adds basic
/// authentication if configured.
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
#[instrument(skip_all)]
pub(crate) async fn setup(
    activity_tracker: DynActivityTracker,
    db: DynDB,
    manual_event_discovery_db: Option<std::sync::Arc<PgDB>>,
    image_storage: DynImageStorage,
    you_com_cfg: Option<YouComConfig>,
    meetings_cfg: Option<MeetingsConfig>,
    payments_cfg: Option<PaymentsConfig>,
    payments_manager: DynPaymentsManager,
    notifications_manager: DynNotificationsManager,
    server_cfg: &HttpServerConfig,
) -> Result<Router> {
    // Check whether the Zoom meetings provider is enabled
    let zoom_enabled = meetings_cfg
        .as_ref()
        .and_then(|cfg| cfg.zoom.as_ref())
        .is_some_and(|zoom_cfg| zoom_cfg.enabled);

    // Check whether a payments provider is configured
    let payments_enabled = payments_cfg.is_some();

    // Setup router state
    let state = State {
        db: db.clone(),
        activity_tracker,
        image_storage,
        manual_event_discovery: you_com_cfg
            .clone()
            .zip(manual_event_discovery_db.clone())
            .map(|(cfg, db)| ManualEventDiscovery::new(cfg, db)),
        manual_job_discovery: you_com_cfg
            .zip(manual_event_discovery_db)
            .map(|(cfg, db)| ManualJobDiscovery::new(cfg, db)),
        meetings_cfg,
        notifications_manager,
        payments_cfg,
        payments_manager,
        serde_qs_de: serde_qs_config(),
        server_cfg: server_cfg.clone(),
    };

    // Setup authentication layer
    let auth_layer = crate::auth::setup_layer(server_cfg, db)?;

    // Setup sub-routers
    let api_router = api::setup_api_router();
    let alliance_dashboard_router = dashboard::setup_alliance_dashboard_router(&state);
    let group_dashboard_router = dashboard::setup_group_dashboard_router(&state);
    let user_dashboard_router = dashboard::setup_user_dashboard_router();

    // Setup router
    // Routes that require login are placed before the login_required middleware layer.
    let mut router = Router::new()
        // Alliance-prefixed protected routes
        .route(
            "/{alliance}/check-in/{event_id}",
            get(event::check_in_page).post(event::check_in),
        )
        .route(
            "/{alliance}/event/{event_id}/attend",
            post(event::attend_event),
        )
        .route(
            "/{alliance}/event/{event_id}/checkout",
            delete(event::cancel_checkout).post(event::start_checkout),
        )
        .route(
            "/{alliance}/event/{event_id}/attendance",
            get(event::attendance_status),
        )
        .route(
            "/{alliance}/event/{event_id}/leave",
            delete(event::leave_event),
        )
        .route(
            "/{alliance}/event/{event_id}/refund-request",
            post(event::request_refund),
        )
        .route(
            "/{alliance}/event/{event_id}/cfs-submissions",
            post(event::submit_cfs_submission),
        )
        .route(
            "/{alliance}/group/{group_id}/cfs-submissions",
            post(group::submit_cfs_submission),
        )
        .route("/{alliance}/group/{group_id}/join", post(group::join_group))
        .route(
            "/{alliance}/group/{group_id}/accelerator/cohorts/{cohort_id}/apply",
            post(group::apply_to_accelerator_cohort),
        )
        .route(
            "/{alliance}/group/{group_id}/accelerator/weeks/{week_id}/updates",
            post(group::submit_accelerator_weekly_update),
        )
        .route(
            "/{alliance}/group/{group_id}/leave",
            delete(group::leave_group),
        )
        .route(
            "/{alliance}/group/{group_id}/membership",
            get(group::membership_status),
        )
        .route(
            "/{alliance}/group/{group_slug}/spotlights",
            get(group::spotlights_page),
        )
        .route(
            "/{alliance}/group/{group_slug}/members",
            get(group::members_page),
        )
        .route(
            "/{alliance}/group/{group_slug}/book-exchange",
            get(group::book_exchange_page),
        )
        .route(
            "/{alliance}/group/{group_slug}/members/{user_id}/phone-requests",
            post(group::request_member_phone),
        )
        .route(
            "/{alliance}/group/{group_slug}/members/{user_id}/mock-interview-requests",
            post(group::request_member_mock_interview),
        )
        .route(
            "/{alliance}/group/{group_slug}/members/phone-requests/{user_id}/approve",
            post(group::approve_member_phone_request),
        )
        .route("/jobs/{job_id}/apply", post(site::jobs::apply))
        .route(
            "/jobs/mock-interviews/request",
            post(site::jobs::request_mock_interview),
        )
        .route(
            "/profiles/{username}/mentorship-requests",
            post(site::profile::request_mentorship),
        )
        .route(
            "/profiles/{username}/coffee-requests",
            post(site::profile::request_coffee),
        )
        // Protected dashboard routes
        .route(
            "/dashboard/account/update/details",
            put(auth::update_user_details),
        )
        .route(
            "/dashboard/account/update/password",
            put(auth::update_user_password),
        )
        .route(
            "/dashboard",
            get(|| async { Redirect::to("/dashboard/user") }),
        )
        .route(
            "/dashboard/jobs",
            get(crate::handlers::dashboard::jobs::page).post(crate::handlers::dashboard::jobs::add),
        )
        .route(
            "/dashboard/jobs/mock-interviews",
            get(crate::handlers::dashboard::jobs::mock_interviews_page),
        )
        .route(
            "/dashboard/jobs/discovery",
            get(crate::handlers::dashboard::jobs::discovery_page),
        )
        .route(
            "/dashboard/jobs/discovery/settings",
            post(crate::handlers::dashboard::jobs::update_discovery),
        )
        .route(
            "/dashboard/jobs/discovery/sources",
            post(crate::handlers::dashboard::jobs::add_discovery_source),
        )
        .route(
            "/dashboard/jobs/discovery/sources/import",
            post(crate::handlers::dashboard::jobs::add_discovery_sources),
        )
        .route(
            "/dashboard/jobs/discovery/sources/{source_id}",
            delete(crate::handlers::dashboard::jobs::delete_discovery_source),
        )
        .route(
            "/dashboard/jobs/discovery/run",
            post(crate::handlers::dashboard::jobs::run_discovery),
        )
        .route(
            "/dashboard/jobs/discovery/items/{item_id}/approve",
            post(crate::handlers::dashboard::jobs::approve_discovery_item),
        )
        .route(
            "/dashboard/jobs/discovery/items/{item_id}/reject",
            post(crate::handlers::dashboard::jobs::reject_discovery_item),
        )
        .route(
            "/dashboard/jobs/mock-interviews/{request_id}/match",
            post(crate::handlers::dashboard::jobs::upsert_mock_interview_match),
        )
        .route(
            "/dashboard/jobs/mock-interviews/matches/{match_id}/feedback",
            post(crate::handlers::dashboard::jobs::update_mock_interview_feedback),
        )
        .route(
            "/dashboard/jobs/users/search",
            get(crate::handlers::dashboard::common::search_user),
        )
        .route(
            "/dashboard/jobs/{job_id}",
            put(crate::handlers::dashboard::jobs::update)
                .delete(crate::handlers::dashboard::jobs::delete),
        )
        .route(
            "/dashboard/jobs/{job_id}/publish",
            put(crate::handlers::dashboard::jobs::publish),
        )
        .route(
            "/dashboard/jobs/{job_id}/unpublish",
            put(crate::handlers::dashboard::jobs::unpublish),
        )
        .nest("/dashboard/alliance", alliance_dashboard_router)
        .nest("/dashboard/group", group_dashboard_router)
        .nest("/dashboard/user", user_dashboard_router)
        // Protected image upload
        .route("/images", post(images::upload))
        .route_layer(login_required!(
            AuthnBackend,
            login_url = LOG_IN_URL,
            redirect_field = "next_url"
        ))
        .nest("/api/v1", api_router)
        // Global site routes (no alliance prefix)
        .route("/", get(site::home::page))
        .route(
            "/apple-touch-icon-precomposed.png",
            get(|| async { StatusCode::NOT_FOUND }),
        )
        .route(
            "/apple-touch-icon.png",
            get(|| async { StatusCode::NOT_FOUND }),
        )
        .route("/explore", get(site::explore::page))
        .route(
            "/explore/events-section",
            get(site::explore::events_section),
        )
        .route(
            "/explore/events-results-section",
            get(site::explore::events_results_section),
        )
        .route(
            "/explore/groups-section",
            get(site::explore::groups_section),
        )
        .route(
            "/explore/groups-results-section",
            get(site::explore::groups_results_section),
        )
        .route("/explore/events/search", get(site::explore::search_events))
        .route("/explore/groups/search", get(site::explore::search_groups))
        .route("/favicon.ico", get(favicon))
        .route("/health-check", get(health_check))
        .route("/robots.txt", get(site::agent_discovery::robots))
        .route("/sitemap.xml", get(site::agent_discovery::sitemap))
        .route("/openapi.yaml", get(site::agent_discovery::openapi))
        .route(
            "/.well-known/api-catalog",
            get(site::agent_discovery::api_catalog),
        )
        .route(
            "/.well-known/mcp/server-card.json",
            get(site::agent_discovery::mcp_server_card),
        )
        .route(
            "/.well-known/agent-skills/index.json",
            get(site::agent_discovery::agent_skills_index),
        )
        .route(
            "/.well-known/agent-skills/goup-mcp/SKILL.md",
            get(site::agent_discovery::mcp_skill),
        )
        .route("/images/og/{file_name}", get(images::serve_open_graph))
        .route("/images/{file_name}", get(images::serve))
        .route("/log-in", get(auth::log_in_page))
        .route("/about", get(site::about::page))
        .route("/docs", get(site::docs::index))
        .route("/docs/{*doc_path}", get(site::docs::page))
        .route("/jobs", get(site::jobs::page))
        .route(
            "/jobs/mock-interviews",
            get(site::jobs::mock_interviews_page),
        )
        .route("/jobs/{slug}", get(site::jobs::details))
        .route("/landscape", get(site::landscape::page))
        .route("/privacy", get(site::privacy::page))
        .route("/profiles/{username}", get(site::profile::page))
        .route("/search", get(site::search::page))
        .route(
            "/sponsor",
            get(site::sponsor::page).post(site::sponsor::submit),
        )
        .route("/stats", get(site::stats::page))
        .route("/wiki", get(site::wiki::page))
        // Alliance-prefixed public routes
        .route("/{alliance}/brand", get(alliance::brand_page))
        .route("/{alliance}/integrations", get(alliance::integrations_page))
        .route("/{alliance}/members", get(alliance::members_page))
        .route("/{alliance}/reports", get(alliance::report_page))
        .route("/{alliance}", get(alliance::page))
        .route(
            "/{alliance}/group/{group_slug}/store",
            get(group::store_page),
        )
        .route(
            "/{alliance}/group/{group_slug}/reports",
            get(group::report_page),
        )
        .route(
            "/{alliance}/group/{group_slug}/accelerator",
            get(group::accelerator_page),
        )
        .route("/{alliance}/group/{group_slug}/cfs", get(group::cfs_page))
        .route("/{alliance}/group/{group_slug}", get(group::page))
        .route(
            "/{alliance}/event/{event_id}/cfs-modal",
            get(event::cfs_modal),
        )
        .route(
            "/{alliance}/group/{group_slug}/event/{event_slug}/availability",
            get(event::availability),
        )
        .route(
            "/{alliance}/group/{group_slug}/event/{event_slug}",
            get(event::page),
        )
        // Page view tracking routes
        .route("/alliances/{alliance_id}/views", post(alliance::track_view))
        .route("/events/{event_id}/views", post(event::track_view))
        .route("/groups/{group_id}/views", post(group::track_view))
        .fallback(site::not_found::page);

    // Setup some routes based on the login options enabled
    if server_cfg.login.email {
        router = router
            .route("/log-in", post(auth::log_in))
            .route("/sign-up", post(auth::sign_up))
            .route("/verify-email/{code}", get(auth::verify_email));
    }
    if server_cfg.login.github {
        router = router
            .route("/log-in/oauth2/{provider}", get(auth::oauth2_redirect))
            .route(
                "/log-in/oauth2/{provider}/callback",
                get(auth::oauth2_callback),
            );
    }
    if server_cfg.login.linkedin {
        router = router
            .route("/log-in/oidc/{provider}", get(auth::oidc_redirect))
            .route("/log-in/oidc/{provider}/callback", get(auth::oidc_callback));
    }

    router = router
        .route("/log-out", get(auth::log_out))
        .route("/section/user-menu", get(auth::user_menu_section))
        .route("/sign-up", get(auth::sign_up_page));

    // Setup Zoom webhook route if enabled in configuration
    if zoom_enabled {
        router = router.route("/webhooks/zoom", post(meetings::zoom_event));
    }

    // Setup the payments webhook route if enabled in configuration
    if payments_enabled {
        router = router.route("/webhooks/payments", post(payments::webhook));
    }

    router = router
        .layer(MessagesManagerLayer)
        .layer(auth_layer)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .route("/static/{*file}", get(static_handler))
        .layer(SetResponseHeaderLayer::if_not_present(
            CACHE_CONTROL,
            HeaderValue::from_static(CACHE_CONTROL_PRIVATE_NO_STORE),
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            redirect_old_hosts,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            route_custom_domains,
        ))
        .layer(middleware::from_fn(add_agent_discovery_links))
        .layer(middleware::from_fn(refresh_stale_clients));

    Ok(router.with_state(state))
}

// Handlers.

/// Redirects favicon requests to the configured site favicon URL.
#[instrument(skip_all)]
async fn favicon(AxumState(db): AxumState<DynDB>) -> impl IntoResponse {
    // Load the configured site settings to resolve the favicon target
    let Ok(site_settings) = db.get_site_settings().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    // Return a plain 404 when no favicon has been configured
    let Some(favicon_url) = site_settings.favicon_url else {
        return StatusCode::NOT_FOUND.into_response();
    };

    // Cache the redirect so browsers avoid repeating this lookup on every visit
    let mut response = Redirect::to(&favicon_url).into_response();
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static(CACHE_CONTROL_FAVICON_REDIRECT),
    );

    response
}

/// Health check endpoint handler.
///
/// Returns 200 OK for monitoring and load balancer health checks.
#[instrument(skip_all)]
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

/// Static file handler for embedded assets.
///
/// Serves files embedded in the binary with appropriate MIME types and cache headers.
#[instrument]
async fn static_handler(uri: Uri) -> impl IntoResponse {
    // Extract file path from URI
    let path = uri.path().trim_start_matches("/static/");

    // Set cache policy based on resource type
    #[cfg(any(not(debug_assertions), test))]
    let cache = if path.starts_with("js/") || path.starts_with("css/") {
        // These assets are hashed.
        CACHE_CONTROL_IMMUTABLE
    } else if path.starts_with("vendor/") {
        // Vendor library files include versions.
        CACHE_CONTROL_IMMUTABLE
    } else if path.starts_with("images/") {
        CACHE_CONTROL_STATIC_IMAGES
    } else {
        // Default cache duration for other static resources.
        CACHE_CONTROL_STATIC_DEFAULT
    };
    #[cfg(all(debug_assertions, not(test)))]
    let cache = CACHE_CONTROL_STATIC_DEVELOPMENT;

    // Get file content and return it (if available)
    match StaticFile::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let headers = [(CONTENT_TYPE, mime.as_ref()), (CACHE_CONTROL, cache)];
            (headers, file.data).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// Middleware.

async fn cached_custom_domain(db: &DynDB, hostname: &str) -> Result<Option<CustomDomainTarget>> {
    if let Some(entry) = CUSTOM_DOMAIN_CACHE.read().await.get(hostname)
        && entry.expires_at > Instant::now()
    {
        return Ok(entry.target.clone());
    }

    let target = db.resolve_active_custom_domain(hostname).await?;
    let now = Instant::now();
    let mut cache = CUSTOM_DOMAIN_CACHE.write().await;
    cache.retain(|_, entry| entry.expires_at > now);
    if cache.len() >= CUSTOM_DOMAIN_CACHE_MAX_ENTRIES
        && let Some(oldest_hostname) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.expires_at)
            .map(|(hostname, _)| hostname.clone())
    {
        cache.remove(&oldest_hostname);
    }
    let ttl = if target.is_some() {
        CUSTOM_DOMAIN_CACHE_TTL
    } else {
        CUSTOM_DOMAIN_NEGATIVE_CACHE_TTL
    };
    cache.insert(
        hostname.to_string(),
        CachedCustomDomain {
            expires_at: now + ttl,
            target: target.clone(),
        },
    );
    Ok(target)
}

fn request_hostname(request: &Request) -> Option<String> {
    request
        .headers()
        .get(HOST)
        .and_then(|header| header.to_str().ok())
        .and_then(|host| host.split(':').next())
        .map(str::to_ascii_lowercase)
}

fn requires_canonical_host(method: &Method, path: &str, target: &CustomDomainTarget) -> bool {
    if !matches!(*method, Method::GET | Method::HEAD) {
        return true;
    }
    if path == "/" || path.starts_with("/static/") || path.starts_with("/images/") {
        return false;
    }

    let target_path = target.canonical_path();
    if path == target_path
        || (target.event_slug.is_some() && path == format!("{target_path}/availability"))
    {
        return false;
    }
    true
}

/// Resolves active group and event hostnames while keeping authenticated flows canonical.
async fn route_custom_domains(
    AxumState(state): AxumState<State>,
    mut request: Request,
    next: Next,
) -> Response {
    request.headers_mut().remove(VERIFIED_CUSTOM_DOMAIN_HEADER);

    let Some(hostname) = request_hostname(&request) else {
        return next.run(request).await;
    };
    #[cfg(test)]
    if hostname == "example.test" {
        return next.run(request).await;
    }
    let canonical_hostname = reqwest::Url::parse(&state.server_cfg.base_url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_ascii_lowercase));
    if canonical_hostname.as_deref() == Some(hostname.as_str()) {
        return next.run(request).await;
    }

    if matches!(
        request.uri().path(),
        "/health-check" | "/robots.txt" | "/favicon.ico"
    ) {
        return next.run(request).await;
    }

    let target = match cached_custom_domain(&state.db, &hostname).await {
        Ok(Some(target)) => target,
        Ok(None) => return StatusCode::MISDIRECTED_REQUEST.into_response(),
        Err(error) => {
            tracing::error!(%error, %hostname, "custom domain lookup failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if requires_canonical_host(request.method(), request.uri().path(), &target) {
        let location = format!(
            "{}{}",
            state.server_cfg.base_url.trim_end_matches('/'),
            request
                .uri()
                .path_and_query()
                .map_or("/", axum::http::uri::PathAndQuery::as_str)
        );
        return Redirect::temporary(&location).into_response();
    }

    request.headers_mut().insert(
        VERIFIED_CUSTOM_DOMAIN_HEADER,
        HeaderValue::from_static("true"),
    );

    if request.uri().path() == "/" {
        let mut path = target.canonical_path();
        if let Some(query) = request.uri().query() {
            path.push('?');
            path.push_str(query);
        }
        match path.parse() {
            Ok(uri) => *request.uri_mut() = uri,
            Err(error) => {
                tracing::error!(%error, %hostname, "custom domain path rewrite failed");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }

    request.extensions_mut().insert(target);
    next.run(request).await
}

/// Middleware that redirects requests from old hosts to the base URL.
///
/// If the request's Host header matches any hostname in the configured `redirect_hosts`
/// list, the request is redirected with a 301 permanent redirect to the base URL.
async fn redirect_old_hosts(
    AxumState(server_cfg): AxumState<HttpServerConfig>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    if let Some(redirect_hosts) = &server_cfg.redirect_hosts
        && let Some(host) = request.headers().get(HOST).and_then(|h| h.to_str().ok())
    {
        // Strip port from host if present
        let host = host.split(':').next().unwrap_or(host);

        // Redirect if host matches any of the redirect hosts
        if redirect_hosts.iter().any(|h| h == host) {
            return Redirect::permanent(&server_cfg.base_url).into_response();
        }
    }
    next.run(request).await.into_response()
}

/// Middleware that refreshes dynamic clients loaded from an older application commit.
async fn refresh_stale_clients(request: Request, next: Next) -> impl IntoResponse {
    let is_htmx = header_value_is_true(request.headers(), "hx-request");
    let is_ocg_fetch = header_value_is_true(request.headers(), "x-ocg-fetch");

    if (is_htmx || is_ocg_fetch) && request_has_stale_commit_sha(request.headers()) {
        return stale_client_refresh_response(is_htmx, is_ocg_fetch);
    }

    let mut response = next.run(request).await.into_response();
    insert_commit_sha_header(response.headers_mut());

    response
}

/// Advertises public machine-readable resources on HTML responses.
async fn add_agent_discovery_links(request: Request, next: Next) -> impl IntoResponse {
    let mut response = next.run(request).await.into_response();
    let is_html = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html"));
    if is_html {
        let headers = response.headers_mut();
        let link = HeaderName::from_static("link");
        for value in [
            "</.well-known/api-catalog>; rel=\"api-catalog\"",
            "</openapi.yaml>; rel=\"service-desc\"; type=\"application/vnd.oai.openapi\"",
            "</docs/api>; rel=\"service-doc\"",
            "</sitemap.xml>; rel=\"sitemap\"; type=\"application/xml\"",
            "</.well-known/mcp/server-card.json>; rel=\"mcp-server-card\"",
            "</.well-known/agent-skills/index.json>; rel=\"agent-skills\"",
        ] {
            headers.append(link.clone(), HeaderValue::from_static(value));
        }
    }
    response
}

/// Returns whether a request header has the string value `true`.
fn header_value_is_true(headers: &axum::http::HeaderMap, header_name: &str) -> bool {
    headers
        .get(header_name)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

/// Inserts the current commit SHA response header.
fn insert_commit_sha_header(headers: &mut axum::http::HeaderMap) {
    headers.insert(
        HeaderName::from_static(COMMIT_SHA_HEADER),
        HeaderValue::from_static(COMMIT_SHA),
    );
}

/// Returns whether the request came from a page loaded with an older commit.
fn request_has_stale_commit_sha(headers: &axum::http::HeaderMap) -> bool {
    headers
        .get(COMMIT_SHA_HEADER)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value != COMMIT_SHA)
}

/// Builds the refresh response returned to stale dynamic clients.
fn stale_client_refresh_response(is_htmx: bool, is_ocg_fetch: bool) -> axum::response::Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    let headers = response.headers_mut();
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static(CACHE_CONTROL_NO_STORE),
    );
    insert_commit_sha_header(headers);

    if is_htmx {
        headers.insert(
            HeaderName::from_static("hx-refresh"),
            HeaderValue::from_static("true"),
        );
    }
    if is_ocg_fetch {
        headers.insert(
            HeaderName::from_static(OCG_REFRESH_HEADER),
            HeaderValue::from_static("true"),
        );
    }

    response
}

// Helpers.

/// Returns the `serde_qs` configuration for query string parsing.
pub(crate) fn serde_qs_config() -> serde_qs::Config {
    serde_qs::Config::new().max_depth(6).use_form_encoding(true)
}
