use std::sync::Arc;

use async_trait::async_trait;
use songbird::{Event, EventContext, EventHandler, TrackEvent};
use tokio::sync::Mutex;

use super::Player;

/// Handler in charge of playing next track in queue after one finishes.
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
        let EventContext::Track([_, ..]) = ctx else { return None; };
        let mut player = self.player.lock().await;
        player.queue.play_load_next();


        None
    }
}
