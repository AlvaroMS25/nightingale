use std::num::NonZeroU64;
use std::ops::Deref;
use axum::body::Body;
use axum::extract::{Query, State as AxumState};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use songbird::input::{Input, YoutubeDl};
use tracing::info;
use uuid::Uuid;
use crate::api::extractors::call::CallExtractor;
use crate::api::extractors::session::SessionExtractor;
use crate::api::model::play::{PlayOptions, PlaySource};
use crate::api::state::State;

const NOT_CONNECTED: &str = r#"{"message": "Not connected to voice"}"#;

#[derive(Deserialize)]
pub struct PlayQuery {
    session: Uuid,
    guild_id: NonZeroU64
}

pub async fn play(
    AxumState(state): AxumState<State>,
    SessionExtractor(session): SessionExtractor,
    Query(query): Query<PlayQuery>,
    Json(options): Json<PlayOptions>
) -> impl IntoResponse {
    info!("Received play request");
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

pub async fn pause(CallExtractor(call): CallExtractor) -> impl IntoResponse {
    let _ = call.lock().await.queue().pause();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

pub async fn resume(CallExtractor(call): CallExtractor) -> impl IntoResponse {
    let _ = call.lock().await.queue().resume();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}
