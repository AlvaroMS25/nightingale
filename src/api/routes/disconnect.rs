use std::num::NonZeroU64;
use axum::body::Body;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use uuid::Uuid;
use crate::api::extractors::session::SessionExtractor;

#[derive(Deserialize)]
pub struct DisconnectVoiceQuery {
    session: Uuid,
    guild_id: NonZeroU64
}

pub async fn disconnect_voice(
    SessionExtractor(session): SessionExtractor,
    Query(query): Query<DisconnectVoiceQuery>
) -> impl IntoResponse {
    let _ = session.read().await.playback.songbird.leave(query.guild_id).await;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}
