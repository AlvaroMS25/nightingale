use std::num::NonZeroU64;
use axum::body::Body;
use axum::extract::{Query, State as AxumState};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use songbird::input::{Input, YoutubeDl};
use uuid::Uuid;
use crate::api::extractors::session::SessionExtractor;
use crate::api::model::play::{PlayOptions, PlaySource};
use crate::api::state::State;

const NOT_CONNECTED: &str = r#"{"message": "Not connected to voice"}"#;

pub struct PlaySourceQuery {
    session: Uuid,
    guild_id: NonZeroU64
}

pub async fn play_source(
    AxumState(state): AxumState<State>,
    SessionExtractor(session): SessionExtractor,
    Query(query): Query<PlaySourceQuery>,
    Json(options): Json<PlayOptions>
) -> impl IntoResponse {
    let source: Input = match options.source {
        PlaySource::Bytes(bytes) => bytes.into(),
        PlaySource::Link(link) => YoutubeDl::new(state.http.clone(), link).into()
    };

    let mut lock = session.write().await;

    let Some(call) = lock.playback.songbird.get(query.guild_id) else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(NOT_CONNECTED))
            .unwrap();
    };

    let handle = call.lock().await.enqueue_input(source).await;

    lock.playback.queue.for_guild(query.guild_id.get()).push(handle);

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}