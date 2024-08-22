pub mod handler;
pub mod queue;

use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;
use songbird::{Config, ConnectionInfo, Driver};
use songbird::error::ConnectionError;
use songbird::id::{ChannelId, GuildId};
use songbird::input::Input;
use songbird::tracks::{Track as SongbirdTrack, TrackHandle};
use tracing::{info, instrument, warn};
use crate::playback::metadata::TrackMetadata;
use crate::api::model::player::Player as PlayerModel;
use queue::Queue;
use crate::api::model::gateway::{Outgoing, OutgoingEvent};
use crate::api::model::play::PlaySource;
use crate::api::model::track::{Track as TrackModel, Track};
use crate::channel::Sender;
use crate::ext::{AsyncIteratorExt, AsyncOptionExt};
use crate::metrics::metrics;
use crate::playback::handle::HandleWithSource;
use crate::playback::player::queue::RepeatMode;
use crate::ptr::SharedPtr;
use crate::source::Sources;

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
    pub paused: bool,
    pub sender: Sender,
    pub sources: SharedPtr<Sources>
}

unsafe impl Send for Player {}

impl Player {
    pub fn new(guild_id: GuildId, sources: SharedPtr<Sources>, config: Config, sender: Sender) -> Self {
        metrics().active_players.inc();
        Self {
            guild_id,
            channel_id: None,
            driver: Driver::new(config),
            queue: Queue::new(),
            volume: 1.0,
            paused: false,
            sender,
            sources
        }

        //handler::PlaybackHandler::register(Arc::clone(&this)).await;
    }

    /// Pauses the current playing track (if exists) and plays the provided one
    /// directly.
    pub async fn play_now<T>(&mut self, item: T, meta: TrackMetadata, source: PlaySource)
    where
        T: Into<Input>
    {
        if self.queue.is_repeat_enabled() {
            self.queue.backup.push_front(source.clone());
        }

        self.queue.pause();
        let handle = self.get_handle(item, meta).await;
        if handle.play().is_err() {
            warn!("Failed to play track directly, resuming queue");
            self.queue.resume();
        }
        self.queue.force_track(HandleWithSource::new(handle, source.into()));
    }

    /// Enqueues the provided input.
    pub async fn enqueue<T>(&mut self, item: T, meta: TrackMetadata, source: PlaySource)
    where
        T: Into<Input>
    {
        if self.enqueue_inner(item, meta, source).await {
            self.play_load_next().await;
        }
    }

    /// Submits the provided input to the call driver, getting a [`TrackHandle`] and
    /// inserting the track data.
    async fn get_handle<T: Into<Input>>(&mut self, item: T, data: TrackMetadata) -> TrackHandle {
        let track = SongbirdTrack::new_with_data(item.into(), Arc::new(data)).volume(self.volume);
        let handle = self.driver.play(track.pause());

        handle
    }

    async fn enqueue_inner<T>(&mut self, item: T, meta: TrackMetadata, source: PlaySource) -> bool
    where
        T: Into<Input>
    {
        if self.queue.is_repeat_enabled() {
            self.queue.backup.push_back(source.clone());
        }

        let handle = self.get_handle(item, meta).await;
        self.queue.enqueue(HandleWithSource::new(handle, source.into()))
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
        metrics().active_players.dec();
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

    pub async fn set_repeat(&mut self, repeat_mode: RepeatMode) {
        self.queue.set_repeat(repeat_mode).await;

        if self.queue.should_repeat_now() {
            self.repeat_queue().await;
        }
    }

    async fn repeat_queue(&mut self) {
        if let RepeatMode::Finite(reps) = &mut self.queue.repeat {
            *reps = reps.saturating_sub(1);

            if *reps == 0 {
                self.queue.repeat = RepeatMode::Off;
            }
        }

        let backup = self.queue.backup.drain(..).collect::<VecDeque<_>>();

        for mut source in backup {
            let playable = match self.sources.playable_for(&mut source).await {
                Ok(p) => p,
                Err(e) => {
                    let _ = self.sender.send(Outgoing::Event {
                        guild_id: self.guild_id.0.get(),
                        event: OutgoingEvent::TrackErrored {
                            error: e.msg,
                            track: if source.is_link() {
                                Track::default()
                            } else {
                                source.track().unwrap_or_default()
                            }
                        }
                    });

                    continue;
                }
            };

            self.enqueue_inner(playable.input, TrackMetadata {
                guild: self.guild_id.0.get(),
                metadata: playable.meta
            }, source).await;
        }
    }

    pub async fn play_load_next(&mut self) {
        // take the track that finished playing.
        self.queue.current.take();

        if self.queue.should_play() {
            // if true here, we're empty of tracks.
            info!("Queue empty");
            metrics().playing_players.dec();
            return;
        }

        while let Err(e) = self.queue.play_next() {
            warn!("Failed to play queued track: {e}");

            if self.queue.should_repeat_now() {
                self.repeat_queue().await;
            }

            if !self.queue.load_next() {
                warn!("Queue finished after having an error playing a track");
                self.queue.current.take();
                return;
            }
        }

        // If we're here, we succeeded on playing, so load next one if available
        if self.queue.should_repeat_now() {
            self.repeat_queue().await;
        }

        self.queue.load_next();
    }

    pub fn as_json(&self) -> PlayerModel {
        fn track(handle: &TrackHandle) -> TrackModel {
            handle.data::<TrackMetadata>().track()
        }

        PlayerModel {
            guild_id: self.guild_id.0,
            channel_id: self.channel_id.map(|c| c.0),
            paused: self.paused,
            volume: (self.volume * 100.0) as _,
            currently_playing: self.queue.current().map(track),
            queue: {
                let mut v = Vec::new();

                if let Some(next) = self.queue.next().map(track) {
                    v.push(next);
                }

                v.extend(self.queue.rest.iter()
                    .map(|t| track(&t.handle)));

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
