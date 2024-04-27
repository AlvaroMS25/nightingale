use std::collections::VecDeque;
use songbird::tracks::{TrackHandle, TrackResult};
use crate::api::model::play::PlaySource;
use crate::playback::handle::HandleWithSource;

#[derive(Default)]
pub enum RepeatMode {
    #[default]
    Off,
    Finite(u32),
    Infinite
}

/// The queue of a player, keeps the next track in queue loaded so skip
/// can happen quickly.
pub struct Queue {
    /// The currently playing track.
    pub current: Option<HandleWithSource>,
    /// The next track in queue.
    pub next: Option<HandleWithSource>,
    /// The rest of the queue.
    pub rest: VecDeque<HandleWithSource>,
    pub backup: VecDeque<PlaySource>,
    pub repeat: RepeatMode,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            current: None,
            next: None,
            rest: VecDeque::new(),
            backup: VecDeque::new(),
            repeat: RepeatMode::Off,
        }
    }

    pub fn should_play(&self) -> bool {
        self.current.is_none() && self.next.is_none() && self.rest.is_empty()
    }

    pub fn current(&self) -> Option<&TrackHandle> {
        self.current.as_ref().map(|i| &i.handle)
    }

    #[allow(unused)]
    pub fn next(&self) -> Option<&TrackHandle> {
        self.next.as_ref().map(|i| &i.handle)
    }

    pub fn pause(&self) {
        self.current().map(|t| t.pause());
    }

    pub fn resume(&self) {
        self.current().map(|t| t.play());
    }

    #[allow(unused)]
    pub fn skip(&mut self) -> Option<TrackResult<TrackHandle>> {
        let current = self.current.take()?;

        // Stopping the current track triggers the TrackEnd event, so the
        // event handler will play the next one.
        Some(current.handle.stop().map(|_| current.handle))
    }

    pub fn set_volume(&self, volume: f32) {
        self.current.as_ref().map(|handle| handle.handle.set_volume(volume));
        self.next.as_ref().map(|handle| handle.handle.set_volume(volume));
    }

    pub async fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat = mode;

        if self.is_repeat_enabled() {
            if !self.backup.is_empty() {
                self.backup.clear(); // clear previous backup queue if exists
            }

            if let Some(current) = self.current.as_ref() {
                self.backup.push_back(current.full_source().await)
            }

            if let Some(next) = self.next.as_ref() {
                self.backup.push_back(next.full_source().await);
            }

            for i in &self.rest {
                self.backup.push_back(i.full_source().await);
            }
        } else {
            self.backup.clear();
            self.backup.shrink_to_fit();
        }
    }

    pub fn is_repeat_enabled(&self) -> bool {
        !matches!(&self.repeat, RepeatMode::Off)
    }

    pub fn should_repeat_now(&self) -> bool {
        self.is_repeat_enabled()
            && self.next.is_none()
            && self.rest.is_empty()
    }

    pub fn play_next(&mut self) -> TrackResult<()> {
        if let Some(next) = self.next.take() {
            let res = next.handle.play();
            self.current = Some(next);
            res
        } else {
            Ok(())
        }
    }

    pub fn load_next(&mut self) -> bool {
        if let Some(next) = self.rest.pop_front() {
            let _ = next.handle.make_playable();
            self.next = Some(next);
            true
        } else {
            false
        }
    }

    pub fn enqueue(&mut self, track: HandleWithSource) -> bool {
        if self.should_play() {
            self.next = Some(track);
            true
        } else if self.next.is_none() && self.rest.is_empty() {
            let _ = track.handle.make_playable();
            self.next = Some(track);
            false
        } else {
            self.rest.push_back(track);
            false
        }
    }

    pub fn force_track(&mut self, track: HandleWithSource) {
        if let Some(next) = self.next.take() {
            self.rest.push_front(next);
        }

        self.next = self.current.take();
        self.current = Some(track);
    }

    pub fn clear(&mut self) {
        self.current.take().map(|t| t.handle.stop());
        self.next.take().map(|t| t.handle.stop());

        for t in self.rest.drain(..) {
            let _ = t.handle.stop();
        }
    }
}
