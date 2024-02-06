use std::num::NonZeroU64;
use axum::body::Body;
use axum::extract::{Path, Query, State as AxumState};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use songbird::input::{AuxMetadata, Compose, Input, YoutubeDl};
use tracing::info;
use crate::api::extractors::player::PlayerExtractor;
use crate::api::extractors::session::SessionExtractor;
use crate::api::model::play::{PlayOptions, PlaySource};
use crate::api::state::State;
use crate::playback::metadata::TrackMetadata;

const NOT_CONNECTED: &str = r#"{"message": "Not connected to voice"}"#;

#[derive(Deserialize)]
pub struct PlayQuery {
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
                        .header(
                            axum::http::header::CONTENT_TYPE,
                            super::super::APPLICATION_JSON
                        )
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

    let Ok(serialized) = serde_json::to_string(&metadata.track()) else {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(
                axum::http::header::CONTENT_TYPE,
                super::super::APPLICATION_JSON
            )
            .body(Body::from(r#"{"message": "Failed to serialize track"}"#))
            .unwrap();
    };

    let Some(player) = session.playback.get_player(query.guild_id) else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header(
                axum::http::header::CONTENT_TYPE,
                super::super::APPLICATION_JSON
            )
            .body(Body::from(NOT_CONNECTED))
            .unwrap();
    };

    player.write().await.enqueue(source, metadata).await;

    Response::builder()
        .status(StatusCode::OK)
        .header(
            axum::http::header::CONTENT_TYPE,
            super::super::APPLICATION_JSON
        )
        .body(Body::from(serialized))
        .unwrap()
}

pub async fn pause(PlayerExtractor(player): PlayerExtractor) -> impl IntoResponse {
    let _ = player.read().await.pause();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

pub async fn resume(PlayerExtractor(player): PlayerExtractor) -> impl IntoResponse {
    let _ = player.read().await.resume();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

pub async fn volume(
    PlayerExtractor(player): PlayerExtractor,
    Path(volume): Path<u8>
) -> impl IntoResponse {
    player.write().await.set_volume(volume);

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}
