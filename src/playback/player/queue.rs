use std::collections::VecDeque;

use songbird::tracks::{Track, TrackHandle, TrackResult};
use tracing::{info, warn};

use crate::playback::metadata::TrackMetadata;

/// The queue of a player, keeps the next track in queue loaded so skip
/// can happen quickly.
pub struct Queue {
    /// The currently playing track.
    pub current: Option<TrackHandle>,
    /// The next track in queue.
    pub next: Option<TrackHandle>,
    /// The rest of the queue.
    pub rest: VecDeque<TrackHandle>
}

impl Queue {
    pub fn new() -> Self {
        Self {
            current: None,
            next: None,
            rest: VecDeque::new()
        }
    }

    pub fn should_play(&self) -> bool {
        self.current.is_none() && self.next.is_none() && self.rest.is_empty()
    }

    pub fn current(&self) -> Option<&TrackHandle> {
        self.current.as_ref()
    }

    pub fn next(&self) -> Option<&TrackHandle> {
        self.next.as_ref()
    }

    pub fn pause(&self) {
        self.current().map(|t| t.pause());
    }

    pub fn resume(&self) {
        self.current().map(|t| t.play());
    }

    pub fn skip(&mut self) -> Option<TrackResult<TrackHandle>> {
        let current = self.current.take()?;

        Some(current.stop().map(|_| current))
    }

    pub fn set_volume(&self, volume: u8) {
        let fmt = (volume / 100) as f32;
        self.current.as_ref().map(|handle| handle.set_volume(fmt));
        self.next.as_ref().map(|handle| handle.set_volume(fmt));
    }

    pub fn play_next(&mut self) -> TrackResult<()> {
        if let Some(next) = self.next.take() {
            let res = next.play();
            self.current = Some(next);
            res
        } else {
            Ok(())
        }
    }

    pub fn load_next(&mut self) -> bool {
        if let Some(next) = self.rest.pop_front() {
            let _ = next.make_playable();
            self.next = Some(next);
            true
        } else {
            false
        }
    }

    pub fn play_load_next(&mut self) {
        // take the track that finished playing.
        self.current.take();

        if self.should_play() {
            // if true here, we're empty of tracks.
            info!("Queue empty");
            return;
        }

        while let Err(e) = self.play_next() {
            warn!("Failed to play queued track: {e}");

            if !self.load_next() {
                warn!("Queue finished after having an error playing a track");
                return;
            }
        }

        // If we're here, we succeeded on playing, so load next one if available
        self.load_next();
    }

    pub fn enqueue(&mut self, track: TrackHandle) {
        if self.should_play() {
            self.next = Some(track);
            self.play_load_next();
        } else if self.next.is_none() && self.rest.is_empty() {
            let _ = track.make_playable();
            self.next = Some(track);

        } else {
            self.rest.push_back(track);
        }
    }

    pub fn clear(&mut self) {
        self.current.take().map(|t| t.stop());
        self.next.take().map(|t| t.stop());

        for t in self.rest.drain(..) {
            let _ = t.stop();
        }
    }
}
