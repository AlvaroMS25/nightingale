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

/// Query used by the [`play`] route, since it uses a [`PlayerExtractor`], this is not the
/// whole query.
#[derive(Deserialize)]
pub struct PlayQuery {
    guild_id: NonZeroU64
}

pub async fn play(
    AxumState(state): AxumState<State>,
    PlayerExtractor(player): PlayerExtractor,
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

    player.lock().await.enqueue(source, metadata).await;

    Response::builder()
        .status(StatusCode::OK)
        .header(
            axum::http::header::CONTENT_TYPE,
            super::super::APPLICATION_JSON
        )
        .body(Body::from(serialized))
        .unwrap()
}

/// Pauses the provided player.
pub async fn pause(PlayerExtractor(player): PlayerExtractor) -> impl IntoResponse {
    let _ = player.lock().await.pause();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

/// Resumes the provided player.
pub async fn resume(PlayerExtractor(player): PlayerExtractor) -> impl IntoResponse {
    let _ = player.lock().await.resume();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

/// Changes the volume of the provided, player, take into account that going above 100 can lead
/// to distortions in the playback.
pub async fn volume(
    PlayerExtractor(player): PlayerExtractor,
    Path(volume): Path<u8>
) -> impl IntoResponse {
    player.lock().await.set_volume(volume);

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}
