use std::num::NonZeroU64;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::extract::State as AxumState;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use songbird::input::{AuxMetadata, Compose, Input, YoutubeDl};
use songbird::tracks::TrackHandle;
use tracing::{info, warn};
use uuid::Uuid;

use crate::api::extractors::player::PlayerExtractor;
use crate::api::extractors::session::{SessionExtractor, SessionWithGuildExtractor};
use crate::api::model::connection::DeserializableConnectionInfo;
use crate::api::model::play::{PlayOptions, PlaySource};
use crate::api::model::player::Player;
use crate::api::model::track::Track;
use crate::api::state::State;
use crate::ext::{AsyncIteratorExt, AsyncOptionExt};
use crate::playback::metadata::TrackMetadata;

/// Retrieves information about the given player.
pub async fn info(PlayerExtractor {player, ..}: PlayerExtractor) -> Json<Player> {
    Json(player.lock().await.as_json().await)
}

/// Tries to connect to the provided channel, this route returns a response immediately,
/// and should not be considered connected until the corresponding `update_state` event is received
/// by the client.
pub async fn connect(
    SessionWithGuildExtractor {session, guild}: SessionWithGuildExtractor,
) -> impl IntoResponse {
    info!("Incoming connection request");
    session.playback.get_or_create(guild, Arc::clone(&session)).await;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

pub async fn disconnect(
    SessionWithGuildExtractor{session, guild}: SessionWithGuildExtractor
) -> impl IntoResponse {
    let _ = session.playback.destroy_player(guild).await;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

pub async fn play(
    AxumState(state): AxumState<State>,
    PlayerExtractor {player, guild}: PlayerExtractor,
    Json(options): Json<PlayOptions>
) -> impl IntoResponse {
    info!("Received play request");
    let (source, metadata): (Input, _) = match options.source {
        PlaySource::Bytes {track, bytes} => {
            (bytes.into(), TrackMetadata {
                metadata: track.into(),
                guild: guild.get()
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
                guild: guild.get()
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

    if options.force_play {
        player.lock().await.play_now(source, metadata).await;
    } else {
        player.lock().await.enqueue(source, metadata).await;
    }


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
pub async fn pause(PlayerExtractor {player, ..}: PlayerExtractor) -> impl IntoResponse {
    let _ = player.lock().await.pause();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

/// Resumes the provided player.
pub async fn resume(PlayerExtractor {player, ..}: PlayerExtractor) -> impl IntoResponse {
    let _ = player.lock().await.resume();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

/// Changes the volume of the provided, player, take into account that going above 100 can lead
/// to distortions in the playback.
pub async fn volume(
    AxumState(state): AxumState<State>,
    Path((session, guild, volume)): Path<(Uuid, NonZeroU64, u8)>
) -> Result<Response, Response> {
    let PlayerExtractor { player, .. } = PlayerExtractor::from_id(session, &state, guild)?;
    player.lock().await.set_volume(volume);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}

pub async fn update(
    PlayerExtractor { player, .. }: PlayerExtractor,
    Json(body): Json<DeserializableConnectionInfo>
) -> impl IntoResponse {


    StatusCode::OK
}
