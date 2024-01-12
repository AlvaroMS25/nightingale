use std::num::NonZeroU64;
use std::time::Duration;
use uuid::Uuid;
use crate::playback::Playback;

pub struct Session {
    pub id: Uuid,
    pub playback: Playback,
    pub options: SessionOptions
}

pub struct SessionOptions {
    pub enable_resume: bool,
    pub timeout: Duration
}

impl Session {
    pub fn new(id: Uuid, shards: u64, user_id: NonZeroU64) -> Self {
        Self {
            id,
            playback: Playback::new(shards, user_id),
            options: SessionOptions {
                enable_resume: true,
                timeout: Duration::from_secs(60)
            }
        }
    }
}
