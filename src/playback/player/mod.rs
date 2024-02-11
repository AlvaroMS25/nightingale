use std::sync::Arc;

use songbird::Call;
use songbird::error::JoinResult;
use songbird::id::GuildId;
use songbird::input::Input;
use songbird::tracks::{Track as SongbirdTrack, TrackHandle};
use tokio::sync::Mutex;
use tracing::warn;
use crate::playback::metadata::TrackMetadata;

mod handler;
mod queue;

use queue::Queue;

/// A player of a guild.
pub struct Player {
    pub guild_id: GuildId,
    /// The call used by the player.
    pub call: Call,
    /// Queue of tracks.
    pub queue: Queue,
    /// Current volume of the player.
    pub volume: u8,
    /// Whether if the player is paused.
    pub paused: bool
}

impl Player {
    pub async fn new(guild_id: GuildId, call: Call) -> Arc<Mutex<Self>> {
        let this = Arc::new(Mutex::new(Self {
            guild_id,
            call,
            queue: Queue::new(),
            volume: 100,
            paused: false
        }));

        handler::PlaybackHandler::register(Arc::clone(&this)).await;

        this
    }

    pub async fn play_now<T: Into<Input>>(&mut self, item: T, meta: TrackMetadata) {
        self.queue.pause();
        if self.get_handle(item, meta).await.play().is_err() {
            warn!("Failed to play track directly, resuming queue");
            self.queue.resume();
        }
    }

    pub async fn enqueue<T: Into<Input>>(&mut self, item: T, meta: TrackMetadata) {
        let handle = self.get_handle(item, meta).await;
        self.queue.enqueue(handle);
    }

    async fn get_handle<T: Into<Input>>(&mut self, item: T, data: TrackMetadata) -> TrackHandle {
        let track = <Input as Into<SongbirdTrack>>::into(item.into()).volume((self.volume / 100) as _);
        let handle = self.call.play(track.pause());
        handle.typemap().write().await.insert::<TrackMetadata>(data);

        handle
    }

    pub async fn destroy(&mut self) -> JoinResult<()> {
        self.queue.clear();
        self.call.remove_all_global_events();
        self.call.leave().await
    }

    pub fn pause(&mut self) {
        self.queue.pause();
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.queue.resume();
        self.paused = false;
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.queue.set_volume(volume);

        self.volume = volume;
    }
}
