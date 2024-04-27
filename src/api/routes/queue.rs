use axum::extract::Query;
use axum::Json;
use axum::response::{IntoResponse, Response};
use songbird::error::TrackResult;
use crate::api::error::IntoResponseError;
use crate::api::extractors::player::PlayerExtractor;
use crate::api::serde::from_string::FromString;
use crate::ext::AsyncOptionExt;
use crate::playback::metadata::TrackMetadata;

pub async fn skip(PlayerExtractor {player, ..} : PlayerExtractor) -> Result<Response, IntoResponseError> {
    Ok(player.lock().await
        .queue
        .skip()
        .transpose()?
        .async_map(|track| async move {
            let read = track.typemap()
                .read().await;

            Json(read.get::<TrackMetadata>().unwrap() // can't fail
                .track()).into_response()
        }).await
        .unwrap_or(().into_response()))
}

pub async fn clear(PlayerExtractor {player, ..} : PlayerExtractor) -> impl IntoResponse
{
    player.lock().await.queue.clear();
}

#[derive(serde::Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum RepeatMode {
    CurrentTrack {
        #[serde(default)]
        times: Option<FromString<u32>>
    },
    Queue {
        times: Option<FromString<u32>>
    },
    None
}

pub async fn repeat(
    PlayerExtractor {player, ..} : PlayerExtractor,
    Query(mode): Query<RepeatMode>
) -> Result<impl IntoResponse, IntoResponseError>
{
    use crate::playback::player::queue::RepeatMode as RepeatQueue;

    match mode {
        RepeatMode::CurrentTrack {times} => {
            player.lock().await
                .queue
                .current()
                .map(move |handle| {
                    if let Some(t) = times {
                        handle.loop_for(t.0 as _)?;
                    } else {
                        handle.enable_loop()?;
                    }

                    TrackResult::Ok(())
                })
                .transpose()?;
        },

        RepeatMode::Queue {times} => player.lock().await
            .set_repeat(if let Some(t) = times {
                RepeatQueue::Finite(t.0)
            } else {
                RepeatQueue::Infinite
            }).await,

        RepeatMode::None => {
            let mut lock = player.lock().await;
            lock.set_repeat(RepeatQueue::Off).await;
            let _ = lock.queue.current().map(|c| c.disable_loop());
        }
    }
    Ok(())
}