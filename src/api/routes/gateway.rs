use std::num::NonZeroU64;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use tracing::{info, warn};
use crate::api::extractors::session::SessionExtractor;

/// Query used on [`connect`], since the route uses a [`SessionExtractor`], this is not
// /// the whole needed query.
#[derive(Deserialize)]
pub struct ConnectQuery {
    guild_id: NonZeroU64,
    channel_id: NonZeroU64
}

/// Tries to connect to the provided channel, this route returns a response immediately,
/// and should not be considered connected until the corresponding `update_state` event is received
/// by the client.
pub async fn connect(
    SessionExtractor(session): SessionExtractor,
    Query(query): Query<ConnectQuery>
) -> impl IntoResponse {
    info!("Incoming connection request");
    tokio::spawn(async move {
        
        match session.playback.join(query.guild_id, query.channel_id, Arc::clone(&session)).await {
            Ok(_) => {
                info!("Connecting voice on guild {} and channel id {}", query.guild_id, query.channel_id);
            },
            Err(error) => {
                warn!("An error occurred when connecting voice on guild {}, error: {}", query.guild_id, error);
                let _ = session.playback.leave(query.guild_id).await;
            }
        }
    });

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

/// Query used on [`disconnect`], since the route uses a [`SessionExtractor`], this is not
/// the whole needed query.
#[derive(Deserialize)]
pub struct DisconnectQuery {
    guild_id: NonZeroU64
}

pub async fn disconnect(
    SessionExtractor(session): SessionExtractor,
    Query(query): Query<DisconnectQuery>
) -> impl IntoResponse {
    let _ = session.playback.leave(query.guild_id).await;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}
