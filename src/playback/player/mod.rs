pub mod handler;
pub mod queue;

use std::fmt;
use songbird::{Config, ConnectionInfo, Driver};
use songbird::error::ConnectionError;
use songbird::id::{ChannelId, GuildId};
use songbird::input::Input;
use songbird::tracks::{Track as SongbirdTrack, TrackHandle};
use tracing::{info, instrument, warn};
use crate::playback::metadata::TrackMetadata;
use crate::api::model::player::Player as PlayerModel;
use queue::Queue;
use crate::api::model::track::Track as TrackModel;
use crate::ext::{AsyncIteratorExt, AsyncOptionExt};

/// A player for a guild.
pub struct Player {
    pub guild_id: GuildId,
    pub channel_id: Option<ChannelId>,
    /// The call used by the player.
    pub driver: Driver,
    /// Queue of tracks.
    pub queue: Queue,
    /// Current volume of the player.
    pub volume: f32,
    /// Whether if the player is paused.
    pub paused: bool
}

unsafe impl Send for Player {}

impl Player {
    pub fn new(guild_id: GuildId, config: Config) -> Self {
        Self {
            guild_id,
            channel_id: None,
            driver: Driver::new(config),
            queue: Queue::new(),
            volume: 1.0,
            paused: false
        }

        //handler::PlaybackHandler::register(Arc::clone(&this)).await;
    }

    /// Pauses the current playing track (if exists) and plays the provided one
    /// directly.
    pub async fn play_now<T: Into<Input>>(&mut self, item: T, meta: TrackMetadata) {
        self.queue.pause();
        let handle = self.get_handle(item, meta).await;
        if handle.play().is_err() {
            warn!("Failed to play track directly, resuming queue");
            self.queue.resume();
        }
        self.queue.force_track(handle);
    }

    /// Enqueues the provided input.
    pub async fn enqueue<T: Into<Input>>(&mut self, item: T, meta: TrackMetadata) {
        let handle = self.get_handle(item, meta).await;
        self.queue.enqueue(handle);
    }

    /// Submits the provided input to the call driver, getting a [`TrackHandle`] and
    /// inserting the track data.
    async fn get_handle<T: Into<Input>>(&mut self, item: T, data: TrackMetadata) -> TrackHandle {
        let track = <Input as Into<SongbirdTrack>>::into(item.into()).volume(self.volume);
        let handle = self.driver.play(track.pause());
        handle.typemap().write().await.insert::<TrackMetadata>(data);

        handle
    }

    pub async fn update(&mut self, info: Option<ConnectionInfo>) -> Result<(), ConnectionError> {
        if let Some(info) = info {
            // The handler will update the channel field when events occur, so don't update it here
            self.driver.connect(info).await
        } else {
            self.driver.leave();
            Ok(())
        }
    }

    /// Destroys the player.
    #[instrument]
    pub async fn destroy(&mut self) -> Result<(), ConnectionError> {
        info!("Destroying player");
        self.update(None).await?;
        self.queue.clear();
        self.driver.remove_all_global_events();
        self.driver.leave();
        Ok(())
    }

    /// Pauses the currently playing track if available.
    pub fn pause(&mut self) {
        self.queue.pause();
        self.paused = true;
    }

    /// Resumes the currently playing track if available.
    pub fn resume(&mut self) {
        self.queue.resume();
        self.paused = false;
    }

    /// Changes the volume of the player.
    pub fn set_volume(&mut self, volume: f32) {
        self.queue.set_volume(volume);

        self.volume = volume;
    }

    pub async fn as_json(&self) -> PlayerModel {
        async fn track(handle: &TrackHandle) -> TrackModel {
            let read = handle.typemap().read().await;

            read.get::<TrackMetadata>()
                .map(|t| t.track())
                .unwrap()
        }

        PlayerModel {
            guild_id: self.guild_id.0,
            channel_id: self.channel_id.map(|c| c.0),
            paused: self.paused,
            volume: (self.volume * 100.0) as _,
            currently_playing: self.queue.current.as_ref().async_map(track).await,
            queue: {
                let mut v = Vec::new();

                if let Some(next) = self.queue.next.as_ref().async_map(track).await {
                    v.push(next);
                }

                v.extend(self.queue.rest.iter().async_map::<_, _, _, Vec<_>>(track).await);

                v
            }

        }
    }
}

impl fmt::Debug for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Player")
            .field("guild", &self.guild_id)
            .finish()
    }
}
