use std::num::NonZeroU64;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::extract::State as AxumState;
use axum::Json;
use axum::response::{IntoResponse, Response};
use tracing::info;
use uuid::Uuid;
use crate::api::error::IntoResponseError;

use crate::api::extractors::player::PlayerExtractor;
use crate::api::extractors::session::SessionWithGuildExtractor;
use crate::api::model::connection::DeserializableConnectionInfo;
use crate::api::model::play::PlayOptions;
use crate::api::model::player::{Player, SeekJson};
use crate::api::model::track::Track;
use crate::api::state::State;
use crate::playback::metadata::TrackMetadata;

/// Retrieves information about the given player.
pub async fn info(PlayerExtractor {player, ..}: PlayerExtractor) -> Json<Player> {
    Json(player.lock().await.as_json())
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
    Json(mut options): Json<PlayOptions>
) -> Result<Json<Track>, IntoResponseError> {
    info!("Received play request");
    let ticket = player.ticket();

    let (source, aux_meta) = state.sources.playable_for(&mut options.source).await
        .map(|playable| (playable.input, playable.meta))?;

    let meta = TrackMetadata {
        guild: guild.get(),
        metadata: aux_meta
    };

    let track = meta.track();

    let mut lock = ticket.wait().await;
    if options.force_play {
        lock.play_now(source, meta, options.source).await;
    } else {
        lock.enqueue(source, meta, options.source).await;
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
    Path((session, guild, volume)): Path<(Uuid, NonZeroU64, u16)>
) -> Result<Response, IntoResponseError> {
    let PlayerExtractor { player, .. } = PlayerExtractor::from_id(session, &state, guild)?;

    if !(0..=512).contains(&volume) {
        return Err(IntoResponseError::new("Volume must be an integer between 0 and 512")
            .with_status(StatusCode::BAD_REQUEST)
        )
    }

    player.lock().await.set_volume((volume as f32) / 100.0);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}

pub async fn seek(
    AxumState(state): AxumState<State>,
    Path((session, guild, millis)): Path<(Uuid, NonZeroU64, u64)>
) -> Result<Response, IntoResponseError>
{
    let PlayerExtractor {player, ..} = PlayerExtractor::from_id(session, &state, guild)?;
    let d = std::time::Duration::from_millis(millis);
    let lock = player.lock().await;

    Ok(if let Some(current) = lock.queue.current() {
        let res = current.seek_async(d).await?;
        Json(SeekJson {
            d: res
        }).into_response()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    })
}
