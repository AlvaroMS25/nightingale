use std::num::NonZeroU64;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::extract::State as AxumState;
use axum::Json;
use axum::response::{IntoResponse, Response};
use songbird::input::Input;
use tracing::info;
use uuid::Uuid;
use crate::api::error::IntoResponseError;

use crate::api::extractors::player::PlayerExtractor;
use crate::api::extractors::session::SessionWithGuildExtractor;
use crate::api::model::connection::DeserializableConnectionInfo;
use crate::api::model::play::{PlayOptions, PlaySource};
use crate::api::model::player::Player;
use crate::api::model::track::Track;
use crate::api::state::State;
use crate::playback::metadata::TrackMetadata;

/// Retrieves information about the given player.
pub async fn info(PlayerExtractor {player, ..}: PlayerExtractor) -> Json<Player> {
    Json(player.lock().await.as_json().await)
}

/// Tries to connect to the provided channel, this route returns a response immediately,
/// and should not be considered connected until the corresponding `update_state` event is received
/// by the client.
pub async fn update(
    SessionWithGuildExtractor {session, guild}: SessionWithGuildExtractor,
    body: Option<Json<DeserializableConnectionInfo>>
) -> Result<Response, IntoResponseError> {
    info!("Incoming connection request");
    let player = session.playback.get_or_create(guild, Arc::clone(&session)).await;

    let info = body.map(|j| j.0.into_songbird(session.playback.user_id.0, guild));

    if let Some(info) = info {
        player.lock().await.update(Some(info)).await?;
    } else {
        session.playback.destroy_player(guild).await?;
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}

pub async fn play(
    AxumState(state): AxumState<State>,
    PlayerExtractor {player, guild}: PlayerExtractor,
    Json(options): Json<PlayOptions>
) -> Result<Json<Track>, IntoResponseError> {
    info!("Received play request");
    let (source, metadata): (Input, _) = match options.source {
        PlaySource::Bytes {track, bytes} => {
            (bytes.into(), TrackMetadata {
                metadata: track.map(Into::into).unwrap_or_default(),
                guild: guild.get()
            })
        },
        other => {
            let source = state.sources.source_for(&other);
            let (url, track) = other.into_inner();

            let mut playable = source.play_url(url).await?;

            if let Some(t) = track {
                playable.meta = t.into();
            }

            (playable.input, TrackMetadata { metadata: playable.meta, guild: guild.get() })
        }
    };

    let track = metadata.track();
    if options.force_play {
        player.lock().await.play_now(source, metadata).await;
    } else {
        player.lock().await.enqueue(source, metadata).await;
    }


    Ok(Json(track))
}

/// Pauses the provided player.
pub async fn pause(PlayerExtractor {player, ..}: PlayerExtractor) -> impl IntoResponse {
    player.lock().await.pause();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

/// Resumes the provided player.
pub async fn resume(PlayerExtractor {player, ..}: PlayerExtractor) -> impl IntoResponse {
    player.lock().await.resume();

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
) -> Result<Response, IntoResponseError> {
    let PlayerExtractor { player, .. } = PlayerExtractor::from_id(session, &state, guild)?;
    player.lock().await.set_volume(volume);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}
