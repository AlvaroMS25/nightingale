use axum::Json;
use axum::response::IntoResponse;
use songbird::tracks::TrackHandle;

use crate::api::extractors::player::PlayerExtractor;
use crate::api::model::player::Player;
use crate::api::model::track::Track;
use crate::ext::{AsyncIteratorExt, AsyncOptionExt};
use crate::playback::metadata::TrackMetadata;

async fn track(handle: &TrackHandle) -> Track {
    let read = handle.typemap().read().await;

    read.get::<TrackMetadata>()
        .map(|t| t.track())
        .unwrap()
}

/// Retrieves information about the given player.
pub async fn player(PlayerExtractor(player): PlayerExtractor) -> Json<Player> {
    let mut lock = player.lock().await;

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