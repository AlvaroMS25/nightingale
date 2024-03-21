use std::num::NonZeroU64;
use std::sync::Arc;
use std::time::Duration;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::extract::State as AxumState;
use axum::Json;
use axum::response::{IntoResponse, Response};
use rusty_ytdl::Video;
use serde::Deserialize;
use songbird::input::{AuxMetadata, Compose, HttpRequest, Input, YoutubeDl};
use songbird::tracks::TrackHandle;
use tracing::{error, info, warn};
use uuid::Uuid;
use crate::api::error::IntoResponseError;

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
pub async fn update(
    SessionWithGuildExtractor {session, guild}: SessionWithGuildExtractor,
    body: Option<Json<DeserializableConnectionInfo>>
) -> Result<Response, IntoResponseError> {
    info!("Incoming connection request");
    let player = session.playback.get_or_create(guild, Arc::clone(&session)).await;

    let info = body.map(|j| j.0.into_songbird(session.playback.user_id.0, guild));

    let mut lock = player.lock().await;

    if let Err(e) = lock.update(info).await {
        return Err(IntoResponseError::new(e));
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
                metadata: track.into(),
                guild: guild.get()
            })
        },
        PlaySource::Rytdlp(link) => {
            use rusty_ytdl::Video;

            let v = Video::new(link).unwrap();
            let mut info = v.get_info().await.unwrap();
            let details = info.video_details;
            let format = info.formats.get(0).unwrap();

            let metadata = AuxMetadata {
                track: Some(details.title.clone()),
                artist: details.author.as_ref().map(|s|s.name.clone()),
                album: None,
                date: Some(details.publish_date),
                channels: format.audio_channels,
                channel: details.author.as_ref().map(|s| s.channel_url.clone()),
                start_time: None,
                duration: Some(Duration::from_secs(details.length_seconds.parse().unwrap())),
                sample_rate: format.audio_sample_rate.as_ref().map(|s| s.parse().unwrap()),
                source_url: Some(details.video_url),
                title: Some(details.title),
                thumbnail: details.thumbnails.get(0).map(|t| t.url.clone())
            };

            let format = rusty_ytdl::choose_format(
                info.formats.as_slice(),
                &rusty_ytdl::VideoOptions {
                    quality: rusty_ytdl::VideoQuality::HighestAudio,
                    filter: rusty_ytdl::VideoSearchOptions::Audio,
                    ..Default::default()
                }
            ).unwrap();
            let req = HttpRequest::new(state.http.clone(), format.url);

            (req.into(), TrackMetadata {
                metadata,
                guild: guild.get()
            })
        },
        PlaySource::Link(link) => {
            let mut ytdl = YoutubeDl::new(state.http.clone(), link);

            let metadata = ytdl.aux_metadata().await?;
            (ytdl.into(), TrackMetadata {
                metadata,
                guild: guild.get()
            })
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
) -> Result<Response, IntoResponseError> {
    let PlayerExtractor { player, .. } = PlayerExtractor::from_id(session, &state, guild)?;
    player.lock().await.set_volume(volume);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}
