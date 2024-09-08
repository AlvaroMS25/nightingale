use std::sync::Arc;

use async_trait::async_trait;
use futures_util::FutureExt;
use songbird::{Event, EventContext, EventHandler, TrackEvent};
use songbird::events::context_data::{ConnectData, DisconnectData};
use songbird::tracks::{TrackHandle, TrackState};
use crate::mutex::TicketedMutex;

use super::Player;

/// Handler in charge of managing a player state and its tracks.
pub struct PlayerHandler {
    player: Arc<TicketedMutex<Player>>
}

impl PlayerHandler {
    pub fn register(player: Arc<TicketedMutex<Player>>) {
        let mut lock = unsafe {
            // This can never be UB since we just created the mutex.
            player.lock().now_or_never().unwrap_unchecked()
        };

        lock.driver.add_global_event(TrackEvent::End.into(), Self {
            player: Arc::clone(&player)
        });
    }
}

#[async_trait]
impl EventHandler for PlayerHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        match ctx {
            EventContext::Track(s) => self.handle_track(s).await,
            EventContext::DriverConnect(s) => self.driver_connect(s).await,
            EventContext::DriverReconnect(s) => self.driver_reconnect(s).await,
            EventContext::DriverDisconnect(s) => self.driver_disconnect(s).await,
            _ => None
        }
    }
}

impl PlayerHandler {
    async fn handle_track(&self, data: &[(&TrackState, &TrackHandle)]) -> Option<Event> {
        let &[(_, handle), ..] = data else { return None; };

        let mut player = self.player.lock().await;

        let Some(current) = &player.queue.current else { return None; };
        let current_id = current.handle.uuid();

        if handle.uuid() != current_id {
            // If the track that ended is not the one we were playing, that means some track had an
            // error while loading, so don't do anything else.
            return None;
        }

        player.play_load_next().await;
        None
    }

    async fn driver_connect(&self, data: &ConnectData<'_>) -> Option<Event> {
        self.player.lock()
            .await
            .channel_id = data.channel_id;
        None
    }

    async fn driver_reconnect(&self, data: &ConnectData<'_>) -> Option<Event> {
        self.player.lock()
            .await
            .channel_id = data.channel_id;
        None
    }

    async fn driver_disconnect(&self, _data: &DisconnectData<'_>) -> Option<Event> {
        self.player.lock()
            .await
            .channel_id = None;
        None
    }
}
