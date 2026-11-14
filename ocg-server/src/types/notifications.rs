//! Notification request types shared by handlers and services.

use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    types::dashboard::alliance::contact::AllianceContactFilters,
    validation::{MAX_LEN_M, MAX_LEN_NOTIFICATION_BODY, trimmed_non_empty},
};

/// Alliance-authored message addressed to matching group team members.
#[derive(Debug, Clone)]
pub(crate) struct AllianceCustomNotificationInput {
    pub actor_user_id: Uuid,
    pub alliance_id: Uuid,
    pub content: CustomNotificationContent,
    pub filters: AllianceContactFilters,
}

/// Subject and body of an organizer-authored notification.
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub(crate) struct CustomNotificationContent {
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_NOTIFICATION_BODY))]
    pub body: String,
    #[serde(alias = "title")]
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub subject: String,
}
