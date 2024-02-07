use std::sync::Arc;

use async_trait::async_trait;
use songbird::{Event, EventContext, EventHandler, TrackEvent};
use tokio::sync::{Mutex, MutexGuard};
use tracing::warn;
use crate::playback::metadata::TrackMetadata;

use super::Player;

pub struct PlaybackHandler {
    player: Arc<Mutex<Player>>
}

impl PlaybackHandler {
    pub async fn register(player: Arc<Mutex<Player>>) {
        let mut lock = player.lock().await;

        lock.call.add_global_event(TrackEvent::End.into(), Self {
            player: Arc::clone(&player)
        });
    }
}

#[async_trait]
impl EventHandler for PlaybackHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let EventContext::Track([(_, h), ..]) = ctx else { return None; };
        let mut player = self.player.lock().await;

        if player.queue.current.as_ref().map(|c| c.uuid() != h.uuid()).unwrap_or(true) {
            // If it was a spontaneous track, continue with ours
            if player.queue.current.as_ref().map(|t| t.play().ok()).flatten().is_some() {
                return None
            }
        }

        player.queue.play_load_next();


        None
    }
}
