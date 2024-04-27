use std::num::NonZeroU64;
use std::time::Duration;
use parking_lot::Mutex;
use uuid::Uuid;
use crate::abort::Abort;
use crate::playback::Playback;
use crate::ptr::SharedPtr;
use crate::source::Sources;

/// A session containing multiple players managed by a client.
pub struct Session {
    pub id: Uuid,
    pub playback: Playback,
    pub options: Mutex<SessionOptions>,
    pub cleanup: Mutex<Option<Abort>>
}

pub struct SessionOptions {
    /// Whether if the session is resumable.
    pub enable_resume: bool,
    /// The time the session has to be resumed.
    pub timeout: Duration
}

impl Session {
    pub fn new(id: Uuid, user_id: NonZeroU64, sources: SharedPtr<Sources>) -> Self {
        Self {
            id,
            playback: Playback::new(user_id, sources),
            options: Mutex::new(SessionOptions {
                enable_resume: true,
                timeout: Duration::from_secs(60)
            }),
            cleanup: Mutex::new(None)
        }
    }

    pub async fn destroy(&self) {
        self.playback.destroy().await;
    }
}
