use songbird::Call;
use songbird::error::JoinResult;
use songbird::input::Input;
use songbird::tracks::{Track as SongbirdTrack, TrackHandle};
use crate::playback::metadata::TrackMetadata;

pub struct Player {
    pub call: Call,
    pub queue: Vec<TrackHandle>,
    pub current: Option<TrackHandle>,
    pub volume: u8,
}

impl Player {
    pub fn new(call: Call) -> Self {
        Self {
            call,
            queue: Vec::new(),
            current: None,
            volume: 100
        }
    }

    pub async fn enqueue<T: Into<Input>>(&mut self, item: T, meta: TrackMetadata) {
        let track = <Input as Into<SongbirdTrack>>::into(item.into()).volume((self.volume / 100) as _);
        let handle = self.call.enqueue(track).await;
        handle.typemap().write().await.insert::<TrackMetadata>(meta);

        if self.current.is_none() {
            self.current = Some(handle);
        } else {
            self.queue.push(handle);
        }
    }

    pub async fn destroy(&mut self) -> JoinResult<()> {
        self.call.leave().await
    }

    pub fn pause(&self) {
        let _ = self.call.queue().pause();
    }

    pub fn resume(&self) {
        let _ = self.call.queue().resume();
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.call.queue().modify_queue(|q| {
            for item in q.iter() {
                let _ = item.set_volume((volume / 100) as f32);
            }
        });

        self.volume = volume;
    }
}
