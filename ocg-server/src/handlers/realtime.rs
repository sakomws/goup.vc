//! WebSocket endpoints for Redis-backed discovery completion notifications.

use axum::{
    Extension,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::Response,
};
use futures_util::StreamExt;
use tower_sessions::Session;

use crate::{
    auth::AuthSession,
    handlers::auth::SELECTED_GROUP_ID_KEY,
    services::realtime::{DiscoveryRealtime, group_channel, user_channel},
};

/// Upgrades an authenticated user's discovery notifications stream.
pub(crate) async fn user_discovery(
    websocket: WebSocketUpgrade,
    auth_session: AuthSession,
    Extension(realtime): Extension<DiscoveryRealtime>,
) -> Result<Response, StatusCode> {
    let user_id = auth_session
        .user
        .as_ref()
        .map(|user| user.user_id)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    upgrade(websocket, realtime, user_channel(user_id)).await
}

/// Upgrades the selected group's discovery notifications stream.
///
/// Group read authorization is applied by the route middleware before this handler.
pub(crate) async fn group_discovery(
    websocket: WebSocketUpgrade,
    session: Session,
    Extension(realtime): Extension<DiscoveryRealtime>,
) -> Result<Response, StatusCode> {
    let group_id = session
        .get(SELECTED_GROUP_ID_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;
    upgrade(websocket, realtime, group_channel(group_id)).await
}

async fn upgrade(
    websocket: WebSocketUpgrade,
    realtime: DiscoveryRealtime,
    channel: String,
) -> Result<Response, StatusCode> {
    let subscription = realtime
        .subscribe(channel)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    Ok(websocket.on_upgrade(move |socket| forward_messages(socket, subscription)))
}

async fn forward_messages(mut socket: WebSocket, mut subscription: redis::aio::PubSub) {
    let mut messages = subscription.on_message();
    while let Some(message) = messages.next().await {
        let Ok(payload) = message.get_payload::<String>() else {
            continue;
        };
        if socket.send(Message::Text(payload.into())).await.is_err() {
            break;
        }
    }
}
