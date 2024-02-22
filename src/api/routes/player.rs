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
use crate::api::model::play::{PlayOptions, PlaySource};
use crate::api::model::player::Player;
use crate::api::model::track::Track;
use crate::api::state::State;
use crate::ext::{AsyncIteratorExt, AsyncOptionExt};
use crate::playback::metadata::TrackMetadata;

/// Retrieves information about the given player.
pub async fn info(PlayerExtractor {player, ..}: PlayerExtractor) -> Json<Player> {
    async fn track(handle: &TrackHandle) -> Track {
        let read = handle.typemap().read().await;

        read.get::<TrackMetadata>()
            .map(|t| t.track())
            .unwrap()
    }

    let lock = player.lock().await;

    Json(Player {
        guild_id: lock.guild_id.0,
        channel_id: lock.call.current_channel().map(|c| c.0),
        paused: lock.paused,
        volume: lock.volume,
        currently_playing: lock.queue.current.as_ref().async_map(track).await,
        queue: {
            let mut v = Vec::new();

            if let Some(next) = lock.queue.next.as_ref().async_map(track).await {
                v.push(next);
            }

            v.extend(lock.queue.rest.iter().async_map::<_, _, _, Vec<_>>(track).await);

            v
        }
    })
}

/// Query used on [`connect`], since the route uses a [`SessionExtractor`], this is not
// /// the whole needed query.
#[derive(Deserialize)]
pub struct ConnectQuery {
    channel_id: NonZeroU64
}

/// Tries to connect to the provided channel, this route returns a response immediately,
/// and should not be considered connected until the corresponding `update_state` event is received
/// by the client.
pub async fn connect(
    SessionWithGuildExtractor {session, guild}: SessionWithGuildExtractor,
    Query(query): Query<ConnectQuery>
) -> impl IntoResponse {
    info!("Incoming connection request");
    tokio::spawn(async move {

        match session.playback.join(guild, query.channel_id, Arc::clone(&session)).await {
            Ok(_) => {
                info!("Connecting voice on guild {} and channel id {}", guild, query.channel_id);
            },
            Err(error) => {
                warn!("An error occurred when connecting voice on guild {}, error: {}", guild, error);
                let _ = session.playback.leave(guild).await;
            }
        }
    });

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

pub async fn disconnect(
    SessionWithGuildExtractor{session, guild}: SessionWithGuildExtractor
) -> impl IntoResponse {
    let _ = session.playback.leave(guild).await;

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
        PlaySource::Bytes(bytes) => {
            (bytes.into(), TrackMetadata {
                metadata: AuxMetadata::default(),
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
    PlayerExtractor {player, ..}: PlayerExtractor,
    Path(volume): Path<u8>
) -> impl IntoResponse {
    player.lock().await.set_volume(volume);

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}
