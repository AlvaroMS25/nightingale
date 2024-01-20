use std::num::NonZeroU64;
use std::ops::Deref;
use axum::body::Body;
use axum::extract::{Path, Query, State as AxumState};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use songbird::input::{AuxMetadata, Compose, Input, YoutubeDl};
use tracing::info;
use uuid::Uuid;
use crate::api::extractors::call::CallExtractor;
use crate::api::extractors::session::SessionExtractor;
use crate::api::model::play::{PlayOptions, PlaySource};
use crate::api::state::State;
use crate::playback::metadata::TrackMetadata;

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
    let (source, metadata): (Input, _) = match options.source {
        PlaySource::Bytes(bytes) => {
            (bytes.into(), TrackMetadata {
                metadata: AuxMetadata::default(),
                guild: query.guild_id.get()
            })
        },
        PlaySource::Link(link) => {
            let mut ytdl = YoutubeDl::new(state.http.clone(), link);

            let metadata = match ytdl.aux_metadata().await {
                Ok(m) => m,
                Err(e) => {
                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(format!(r#"{{"message": "{e}"}}"#)))
                        .unwrap();
                }
            };
            (ytdl.into(), TrackMetadata {
                metadata,
                guild: query.guild_id.get()
            })
        }
    };

    let mut lock = session.write().await;

    let Some(call) = lock.playback.get_call(query.guild_id) else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(NOT_CONNECTED))
            .unwrap();
    };

    let handle = call.write().await.enqueue_input(source).await;

    handle.typemap().write().await.insert::<TrackMetadata>(metadata);

    lock.playback.queue.for_guild(query.guild_id.get()).push(handle);

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

pub async fn pause(CallExtractor(call): CallExtractor) -> impl IntoResponse {
    let _ = call.read().await.queue().pause();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

pub async fn resume(CallExtractor(call): CallExtractor) -> impl IntoResponse {
    let _ = call.read().await.queue().resume();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

pub async fn volume(
    CallExtractor(call): CallExtractor,
    Path(volume): Path<f32>
) -> impl IntoResponse {
    call.read().await.queue().modify_queue(|q| {
        for item in q.iter() {
            let _ = item.set_volume(volume);
        }
    });

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}
