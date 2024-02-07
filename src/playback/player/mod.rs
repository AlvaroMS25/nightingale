use std::sync::Arc;

use songbird::Call;
use songbird::error::JoinResult;
use songbird::input::Input;
use songbird::tracks::{Track as SongbirdTrack, TrackHandle};
use tokio::sync::Mutex;
use crate::playback::metadata::TrackMetadata;

mod handler;
mod queue;

use queue::Queue;

pub struct Player {
    pub call: Call,
    pub queue: Queue,
    pub volume: u8,
    pub paused: bool
}

impl Player {
    pub async fn new(call: Call) -> Arc<Mutex<Self>> {
        let this = Arc::new(Mutex::new(Self {
            call,
            queue: Queue::new(),
            volume: 100,
            paused: false
        }));

        handler::PlaybackHandler::register(Arc::clone(&this)).await;

        this
    }

    pub async fn enqueue<T: Into<Input>>(&mut self, item: T, meta: TrackMetadata) {
        let track = <Input as Into<SongbirdTrack>>::into(item.into()).volume((self.volume / 100) as _);
        let handle = self.call.play(track.pause());
        handle.typemap().write().await.insert::<TrackMetadata>(meta);

        self.queue.enqueue(handle);
    }

    pub async fn destroy(&mut self) -> JoinResult<()> {
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
