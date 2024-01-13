use std::num::NonZeroU64;
use axum::body::Body;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use tracing::{info, warn};
use uuid::Uuid;
use crate::api::extractors::session::SessionExtractor;

#[derive(Deserialize)]
pub struct ConnectQuery {
    guild_id: NonZeroU64,
    channel_id: NonZeroU64
}

pub async fn connect(
    SessionExtractor(session): SessionExtractor,
    Query(query): Query<ConnectQuery>
) -> impl IntoResponse {
    info!("Incoming connection request");
    tokio::spawn(async move {
        let mut lock = session.read().await;
        match lock.playback.songbird.join(query.guild_id, query.channel_id).await {
            Ok(_) => {
                info!("Connecting voice on guild {} and channel id {}", query.guild_id, query.channel_id);
            },
            Err(error) => {
                warn!("An error occurred when connecting voice on guild {}, error: {}", query.guild_id, error);
                let _ = lock.playback.songbird.leave(query.guild_id).await;
            }
        }
    });
    info!("Sending response");

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

#[derive(Deserialize)]
pub struct DisconnectQuery {
    guild_id: NonZeroU64
}

pub async fn disconnect(
    SessionExtractor(session): SessionExtractor,
    Query(query): Query<DisconnectQuery>
) -> impl IntoResponse {
    let _ = session.read().await.playback.songbird.remove(query.guild_id).await;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}
