//! Shared domain types exchanged between database, templates, services, and handlers.

use std::{str::FromStr, time::Duration};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnNull, DurationSecondsWithFrac, serde_as, skip_serializing_none};
use sha2::{Digest, Sha256};
use strum::{AsRefStr, Display, EnumString};
use uuid::Uuid;

use crate::auth::User;

/// Required height for Open Graph preview images.
pub const OPEN_GRAPH_IMAGE_HEIGHT: u32 = 630;

/// Required width for Open Graph preview images.
pub const OPEN_GRAPH_IMAGE_WIDTH: u32 = 1200;

/// Geographic bounding box coordinates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BBox {
    pub ne_lat: f64,
    pub ne_lon: f64,
    pub sw_lat: f64,
    pub sw_lon: f64,
}

/// Type alias for result counts, used in pagination.
pub type Total = usize;

/// Enum representing unique identifiers for each page in the application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageId {
    CheckIn,
    AllianceDashboard,
    Alliance,
    Event,
    Group,
    GroupDashboard,
    JobsDashboard,
    OpportunitiesDashboard,
    LogIn,
    SignUp,
    SiteAbout,
    SiteDocs,
    SiteExplore,
    SiteHome,
    SiteJobs,
    SiteOpportunities,
    SiteLandscape,
    SiteNotFound,
    SitePrivacy,
    SiteSearch,
    SiteSponsor,
    SiteStats,
    SiteWiki,
    UserDashboard,
}

/// Parsed and authenticated API caller.
#[derive(Debug, Clone)]
pub struct ApiUser {
    /// Authenticated user.
    pub user: User,
    /// Token scopes granted to the caller.
    pub scopes: Vec<ApiScope>,
}

impl ApiUser {
    /// Returns whether the caller holds the given scope or platform admin access.
    pub fn has_scope(&self, scope: ApiScope) -> bool {
        self.scopes.contains(&scope) || self.scopes.contains(&ApiScope::AdminPlatform)
    }

    /// Current user id.
    pub fn user_id(&self) -> Uuid {
        self.user.user_id
    }
}

/// API token scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApiScope {
    /// Public read endpoints.
    #[serde(rename = "read:public")]
    ReadPublic,
    /// User profile writes.
    #[serde(rename = "write:profile")]
    WriteProfile,
    /// Event actions and writes.
    #[serde(rename = "write:events")]
    WriteEvents,
    /// Job actions and writes.
    #[serde(rename = "write:jobs")]
    WriteJobs,
    /// Alliance scoped admin actions.
    #[serde(rename = "admin:alliance")]
    AdminAlliance,
    /// Platform admin actions.
    #[serde(rename = "admin:platform")]
    AdminPlatform,
}

impl ApiScope {
    /// Scope as persisted text.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadPublic => "read:public",
            Self::WriteProfile => "write:profile",
            Self::WriteEvents => "write:events",
            Self::WriteJobs => "write:jobs",
            Self::AdminAlliance => "admin:alliance",
            Self::AdminPlatform => "admin:platform",
        }
    }
}

impl FromStr for ApiScope {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "read:public" => Ok(Self::ReadPublic),
            "write:profile" => Ok(Self::WriteProfile),
            "write:events" => Ok(Self::WriteEvents),
            "write:jobs" => Ok(Self::WriteJobs),
            "admin:alliance" => Ok(Self::AdminAlliance),
            "admin:platform" => Ok(Self::AdminPlatform),
            _ => Err(()),
        }
    }
}

/// Hash a bearer token for storage and lookup.
pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// Non-secret token metadata returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    /// Token identifier.
    pub api_token_id: Uuid,
    /// Token owner.
    pub user_id: Uuid,
    /// Human-readable name.
    pub name: Option<String>,
    /// Non-secret token prefix.
    pub token_prefix: String,
    /// Token scopes.
    pub scopes: Vec<ApiScope>,
    /// Creation timestamp.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    /// Last-used timestamp.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub last_used_at: Option<DateTime<Utc>>,
    /// Revocation timestamp.
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub revoked_at: Option<DateTime<Utc>>,
}

/// User search result.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DashboardUser {
    pub user_id: Uuid,
    pub username: String,

    pub name: Option<String>,
    pub photo_url: Option<String>,
}

/// Private intentional dating opt-in visible only to authorized admins.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IntentionalDatingOptIn {
    pub user_id: Uuid,
    pub username: String,
    pub group_id: Uuid,
    pub group_name: String,
    pub alliance_id: Uuid,
    pub alliance_display_name: String,

    pub city: Option<String>,
    pub company: Option<String>,
    pub country: Option<String>,
    pub email: Option<String>,
    pub intentional_dating_goals: Option<String>,
    pub intentional_dating_preferences: Option<String>,
    pub name: Option<String>,
    pub photo_url: Option<String>,
    pub title: Option<String>,
}

/// Book exchange member list visible to authorized viewers.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BookExchangeMember {
    pub user_id: Uuid,
    pub username: String,
    pub group_id: Uuid,
    pub group_name: String,
    pub alliance_id: Uuid,
    pub alliance_display_name: String,

    pub book_exchange_books: Option<String>,
    pub city: Option<String>,
    pub company: Option<String>,
    pub country: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub photo_url: Option<String>,
    pub title: Option<String>,
}

/// A persisted event attendee email schedule for organizer management.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScheduledEventAttendeeEmail {
    /// Email body.
    pub body: String,
    /// Number of eligible recipients when last scheduled or sent.
    pub recipient_count: usize,
    /// Recipient scope, either all or selected attendees.
    pub recipient_scope: String,
    /// Explicit selected attendee IDs, if applicable.
    pub recipient_user_ids: Option<Vec<Uuid>>,
    /// Time at which the email is scheduled to send.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub scheduled_at: DateTime<Utc>,
    /// Unique identifier for the schedule.
    pub scheduled_event_attendee_email_id: Uuid,
    /// Delivery lifecycle status.
    pub status: String,
    /// Email subject.
    pub subject: String,
}

/// Image returned from a storage provider.
#[derive(Debug, Clone)]
pub struct Image {
    /// Image contents.
    pub bytes: Vec<u8>,
    /// MIME type set when the image was retrieved.
    pub content_type: String,
}

/// Image to be uploaded to a storage provider.
pub struct NewImage<'a> {
    /// Image contents.
    pub bytes: &'a [u8],
    /// MIME type determined for the image.
    pub content_type: &'a str,
    /// Target file name (hash plus extension).
    pub file_name: &'a str,
    /// Identifier of the user that uploaded the image.
    pub user_id: Uuid,
}

/// Represents a meeting to be synced with the provider.
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Meeting {
    #[serde(alias = "meeting_provider_id", default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub provider: MeetingProvider,

    pub delete: Option<bool>,
    #[serde(alias = "duration_secs")]
    #[serde_as(deserialize_as = "Option<DurationSecondsWithFrac<f64>>")]
    pub duration: Option<Duration>,
    pub event_id: Option<Uuid>,
    pub hosts: Option<Vec<String>>,
    pub join_url: Option<String>,
    pub meeting_id: Option<Uuid>,
    pub password: Option<String>,
    pub provider_host_user_id: Option<String>,
    pub provider_meeting_id: Option<String>,
    #[serde(alias = "meeting_recording_requested")]
    pub recording_requested: Option<bool>,
    pub session_id: Option<Uuid>,
    pub starts_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub sync_claimed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub sync_state_hash: Option<String>,
    pub timezone: Option<String>,
    pub topic: Option<String>,
}

impl Meeting {
    /// Returns the action to take to sync this meeting with the provider.
    pub fn sync_action(&self) -> SyncAction {
        if self.delete == Some(true) {
            SyncAction::Delete
        } else if self.provider_meeting_id.is_none() {
            SyncAction::Create
        } else {
            SyncAction::Update
        }
    }

    /// Returns the end timestamp.
    pub fn ends_at(&self) -> Option<DateTime<Utc>> {
        let starts_at = self.starts_at?;
        let duration = self.duration?;
        let duration = chrono::Duration::from_std(duration).ok()?;

        starts_at.checked_add_signed(duration)
    }
}

/// Meeting provider options.
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    EnumString,
    Eq,
    Hash,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum MeetingProvider {
    #[serde(rename = "google_meet")]
    #[strum(serialize = "google_meet")]
    GoogleMeet,
    #[default]
    Zoom,
}

/// Outcome stored after checking an overdue meeting for automatic ending.
#[derive(AsRefStr, Clone, Copy, Debug, Display, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum MeetingAutoEndCheckOutcome {
    AlreadyNotRunning,
    AutoEnded,
    Error,
    NotFound,
}

/// Result returned by providers when trying to end a meeting.
pub enum MeetingEndResult {
    AlreadyNotRunning,
    Ended,
}

/// Action to take to sync a meeting with the provider.
pub enum SyncAction {
    Create,
    Delete,
    Update,
}

/// Represents a file that should be sent with a notification.
#[derive(Debug, Clone)]
pub struct Attachment {
    /// MIME type for the attachment body.
    pub content_type: String,
    /// Raw attachment data.
    pub data: Vec<u8>,
    /// File name shown to recipients.
    pub file_name: String,
}

/// Data required to create a new notification.
#[derive(Debug, Clone)]
pub struct NewNotification {
    /// Files to include in the notification email.
    pub attachments: Vec<Attachment>,
    /// The type of notification to send.
    pub kind: NotificationKind,
    /// The user IDs to notify.
    pub recipients: Vec<Uuid>,

    /// Optional template data for the notification content.
    pub template_data: Option<serde_json::Value>,
}

/// Data required to send an email outside the user notification queue.
#[derive(Debug, Clone)]
pub struct OutboundEmail {
    /// Plain text email body.
    pub body: String,
    /// Email subject.
    pub subject: String,
    /// Recipient email address.
    pub to: String,
}

/// Data required to deliver a notification to a user.
#[derive(Debug, Clone)]
pub struct Notification {
    /// Files included with the notification.
    pub attachments: Vec<Attachment>,
    /// Email address to send the notification to.
    pub email: String,
    /// The type of notification.
    pub kind: NotificationKind,
    /// Unique identifier for the notification.
    pub notification_id: Uuid,

    /// Optional template data for the notification content.
    pub template_data: Option<serde_json::Value>,
}

/// Supported notification types.
#[derive(Debug, Clone, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum NotificationKind {
    AllianceCustom,
    CfsSubmissionUpdated,
    CoffeeMeetSuggestion,
    DistributionContentDue,
    OpportunityDigest,
    AllianceTeamInvitation,
    EmailVerification,
    EventAttendanceCanceled,
    EventCanceled,
    EventCohostInvitation,
    EventCustom,
    EventInvitation,
    EventPublished,
    EventRefundApproved,
    EventRefundRejected,
    EventRefundRequested,
    EventReminder,
    EventSurveyRequest,
    EventSurveyReminder,
    EventRescheduled,
    EventSeriesCanceled,
    EventSeriesPublished,
    EventWaitlistJoined,
    EventWaitlistLeft,
    EventWaitlistPromoted,
    EventWelcome,
    GroupCustom,
    GroupTeamInvitation,
    GroupWelcome,
    InboxMessageReceived,
    InboxReplyReceived,
    IntentionalDatingIntroduction,
    MockInterviewMatched,
    SiteOnboarding,
    SessionProposalCoSpeakerInvitation,
    SpeakerSeriesWelcome,
    SpeakerWelcome,
}

/// Result returned after creating a checkout session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CheckoutSession {
    /// Provider-specific checkout session identifier.
    pub provider_session_id: String,
    /// Redirect URL for the attendee.
    pub redirect_url: String,
}

/// Alliance member summary information.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceMember {
    /// User identifier.
    pub user_id: Uuid,
    /// Username.
    pub username: String,
    /// Group names this member belongs to in the alliance.
    pub group_names: Vec<String>,

    /// Company the user represents.
    pub company: Option<String>,
    /// User biography.
    pub bio: Option<String>,
    /// Bluesky profile URL.
    pub bluesky_url: Option<String>,
    /// City where the user is based.
    pub city: Option<String>,
    /// Whether this member accepts direct `CoffeeMeet` requests.
    #[serde(default = "default_true")]
    pub coffee_meet_enabled: bool,
    /// Country where the user is based.
    pub country: Option<String>,
    /// Facebook profile URL.
    pub facebook_url: Option<String>,
    /// GitHub profile URL.
    pub github_url: Option<String>,
    /// User interests.
    pub interests: Option<Vec<String>>,
    /// `LinkedIn` profile URL.
    pub linkedin_url: Option<String>,
    /// Whether the user has a connected `LinkedIn` provider.
    pub linkedin_connected: bool,
    /// Whether this member offers mentorship services for businesses.
    #[serde(default)]
    pub mentorship_businesses: bool,
    /// Whether this member offers mentorship services for individuals.
    #[serde(default)]
    pub mentorship_individuals: bool,
    /// Optional mentorship description.
    pub mentorship_note: Option<String>,
    /// Optional price or pricing guidance for mentorship.
    pub mentorship_price: Option<String>,
    /// Full name.
    pub name: Option<String>,
    /// URL to user's avatar.
    pub photo_url: Option<String>,
    /// `Substack` publication URL.
    pub substack_url: Option<String>,
    /// Title held by the user.
    pub title: Option<String>,
    /// X/Twitter profile URL.
    pub twitter_url: Option<String>,
    /// Website URL.
    pub website_url: Option<String>,
    /// `YouTube` channel URL.
    pub youtube_url: Option<String>,
}

fn default_true() -> bool {
    true
}
