//! Core authentication data types shared across crates.

use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::{
    types::user::UserProvider,
    validation::{
        MAX_LEN_DISPLAY_NAME, MAX_LEN_S, MIN_PASSWORD_LEN, trimmed_non_empty, trimmed_non_empty_opt,
    },
};

/// Key used to store the authentication provider in the session.
pub const AUTH_PROVIDER_KEY: &str = "auth_provider";

/// Represents a user in the system.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct User {
    /// Unique user ID.
    pub user_id: Uuid,
    /// Authentication hash for session validation.
    pub auth_hash: String,
    /// User's email address.
    pub email: String,
    /// Whether the user's email is verified.
    pub email_verified: bool,
    /// User's display name.
    pub name: String,
    /// Whether this request just created or activated the user account.
    #[serde(default, skip_serializing)]
    pub newly_registered: bool,
    /// Whether the user receives optional notifications.
    pub optional_notifications_enabled: bool,
    /// Whether the user can manage platform-level resources.
    #[serde(default)]
    pub platform_admin: bool,
    /// Registration state for placeholder and regular users.
    #[serde(default = "default_registration_status")]
    pub registration_status: String,
    /// User's username.
    pub username: String,

    /// Whether the user belongs to any group team.
    pub belongs_to_any_group_team: Option<bool>,
    /// Whether the user belongs to their alliance team.
    pub belongs_to_alliance_team: Option<bool>,
    /// User's biography.
    pub bio: Option<String>,
    /// User's Bluesky URL.
    pub bluesky_url: Option<String>,
    /// Whether the user privately opts into book exchange.
    #[serde(default)]
    pub book_exchange_enabled: bool,
    /// Private book list visible only to eligible community admins.
    pub book_exchange_books: Option<String>,
    /// User's city.
    pub city: Option<String>,
    /// Whether this user accepts direct `CoffeeMeet` requests.
    #[serde(default = "default_true")]
    pub coffee_meet_enabled: bool,
    /// User's company.
    pub company: Option<String>,
    /// User's country.
    pub country: Option<String>,
    /// User's Facebook URL.
    pub facebook_url: Option<String>,
    /// User's GitHub URL.
    pub github_url: Option<String>,
    /// Whether the user has a password set.
    pub has_password: Option<bool>,
    /// User's interests.
    pub interests: Option<Vec<String>>,
    /// Whether the user privately opts into intentional dating introductions.
    #[serde(default)]
    pub intentional_dating_enabled: bool,
    /// Private dating goals visible only to eligible community admins.
    pub intentional_dating_goals: Option<String>,
    /// Private dating preferences visible only to eligible community admins.
    pub intentional_dating_preferences: Option<String>,
    /// User's `LinkedIn` URL.
    pub linkedin_url: Option<String>,
    /// Whether the user offers mentorship services for businesses.
    #[serde(default)]
    pub mentorship_businesses: bool,
    /// Whether the user offers mentorship services for individuals.
    #[serde(default)]
    pub mentorship_individuals: bool,
    /// Optional description of the user's mentorship offering.
    pub mentorship_note: Option<String>,
    /// Optional price or pricing guidance for mentorship.
    pub mentorship_price: Option<String>,
    /// User's password hash (if present).
    pub password: Option<String>,
    /// User's photo URL.
    pub photo_url: Option<String>,
    /// International calling code for the user's phone number.
    pub phone_country_code: Option<String>,
    /// User's phone number.
    pub phone_number: Option<String>,
    /// External provider metadata.
    pub provider: Option<UserProvider>,
    /// User's `Substack` URL.
    pub substack_url: Option<String>,
    /// User's timezone.
    pub timezone: Option<String>,
    /// User's title.
    pub title: Option<String>,
    /// User's Twitter URL.
    pub twitter_url: Option<String>,
    /// User's website URL.
    pub website_url: Option<String>,
    /// User's `YouTube` URL.
    pub youtube_url: Option<String>,
}

impl User {
    /// Returns whether the user has completed the profile fields useful to organizers.
    pub fn is_profile_complete(&self) -> bool {
        profile_field_is_filled(self.title.as_deref())
            && profile_field_is_filled(self.bio.as_deref())
            && [
                self.linkedin_url.as_deref(),
                self.twitter_url.as_deref(),
                self.website_url.as_deref(),
            ]
            .into_iter()
            .any(profile_field_is_filled)
    }
}

fn default_true() -> bool {
    true
}

fn profile_field_is_filled(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

impl axum_login::AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.user_id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.auth_hash.as_bytes()
    }
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("user_id", &self.user_id)
            .field("username", &self.username)
            .finish_non_exhaustive()
    }
}

/// Summary of user information.
#[skip_serializing_none]
#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct UserSummary {
    /// User's email address.
    #[garde(email)]
    pub email: String,
    /// User's display name.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_DISPLAY_NAME))]
    pub name: String,
    /// User's username.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_S))]
    pub username: String,

    /// User's profile photo URL.
    #[garde(skip)]
    pub photo_url: Option<String>,
    /// Whether the user has a password set.
    #[garde(skip)]
    pub has_password: Option<bool>,
    /// User's password (if present).
    #[garde(custom(trimmed_non_empty_opt), length(min = MIN_PASSWORD_LEN, max = MAX_LEN_S))]
    pub password: Option<String>,
    /// External provider metadata.
    #[garde(skip)]
    pub provider: Option<UserProvider>,
}

impl From<User> for UserSummary {
    fn from(user: User) -> Self {
        Self {
            email: user.email,
            name: user.name,
            username: user.username,
            photo_url: user.photo_url,
            has_password: user.has_password,
            password: None,
            provider: user.provider,
        }
    }
}

impl std::fmt::Debug for UserSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserSummary")
            .field("email", &self.email)
            .field("name", &self.name)
            .field("username", &self.username)
            .finish_non_exhaustive()
    }
}

/// Default persisted registration status for regular users.
pub fn default_registration_status() -> String {
    "registered".to_string()
}
