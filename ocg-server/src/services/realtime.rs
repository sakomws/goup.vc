//! Redis Pub/Sub transport for discovery completion notifications.

use std::sync::OnceLock;

use redis::AsyncCommands;
use serde::Serialize;
use tracing::warn;
use uuid::Uuid;

use crate::config::RedisConfig;

static DISCOVERY_REALTIME: OnceLock<DiscoveryRealtime> = OnceLock::new();

/// Optional realtime delivery backed by Redis Pub/Sub.
#[derive(Clone)]
pub(crate) struct DiscoveryRealtime {
    client: redis::Client,
}

impl DiscoveryRealtime {
    /// Builds a realtime transport when Redis is configured.
    pub(crate) fn from_config(config: Option<&RedisConfig>) -> Option<Self> {
        let config = config?;
        match redis::Client::open(config.url.as_str()) {
            Ok(client) => Some(Self { client }),
            Err(error) => {
                warn!(%error, "Redis realtime delivery is disabled due to an invalid Redis URL");
                None
            }
        }
    }

    /// Publishes a completion notification without affecting discovery outcomes.
    pub(crate) async fn publish(&self, notification: DiscoveryCompletion) {
        let channel = notification.channel();
        let payload = match serde_json::to_string(&notification) {
            Ok(payload) => payload,
            Err(error) => {
                warn!(%error, "could not serialize discovery completion notification");
                return;
            }
        };

        let result = async {
            let mut connection = self.client.get_multiplexed_async_connection().await?;
            let _: () = connection.publish(channel, payload).await?;
            redis::RedisResult::Ok(())
        }
        .await;

        if let Err(error) = result {
            warn!(%error, "could not publish discovery completion notification");
        }
    }

    /// Opens a Redis Pub/Sub subscription for a websocket client.
    pub(crate) async fn subscribe(
        &self,
        channel: String,
    ) -> redis::RedisResult<redis::aio::PubSub> {
        let mut pubsub = self.client.get_async_pubsub().await?;
        pubsub.subscribe(channel).await?;
        Ok(pubsub)
    }
}

/// Installs the process-wide optional realtime transport for request-spawned jobs.
pub(crate) fn configure(realtime: Option<DiscoveryRealtime>) {
    if let Some(realtime) = realtime {
        let _ = DISCOVERY_REALTIME.set(realtime);
    }
}

/// Returns the configured realtime transport, when Redis delivery is enabled.
pub(crate) fn configured() -> Option<DiscoveryRealtime> {
    DISCOVERY_REALTIME.get().cloned()
}

/// A discovery run completion payload sent to the owning dashboard scope.
#[derive(Debug, Serialize)]
#[serde(tag = "scope", rename_all = "snake_case")]
pub(crate) enum DiscoveryCompletion {
    /// Completion for one user's jobs discovery run.
    User {
        user_id: Uuid,
        status: DiscoveryStatus,
        discovered_count: i32,
        created_count: i32,
    },
    /// Completion for one selected group's event discovery run.
    Group {
        group_id: Uuid,
        status: DiscoveryStatus,
        discovered_count: i32,
        created_count: i32,
    },
}

impl DiscoveryCompletion {
    /// Returns the private Redis channel for this notification's scope.
    pub(crate) fn channel(&self) -> String {
        match self {
            Self::User { user_id, .. } => user_channel(*user_id),
            Self::Group { group_id, .. } => group_channel(*group_id),
        }
    }
}

/// The terminal state of a discovery run.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DiscoveryStatus {
    /// The run completed successfully.
    Succeeded,
    /// The run completed with an error.
    Failed,
}

/// Returns the Redis channel for one user's discovery notifications.
pub(crate) fn user_channel(user_id: Uuid) -> String {
    format!("ocg:discovery:user:{user_id}")
}

/// Returns the Redis channel for one group's discovery notifications.
pub(crate) fn group_channel(group_id: Uuid) -> String {
    format!("ocg:discovery:group:{group_id}")
}
