use std::num::NonZeroU64;
use std::time::Duration;
use parking_lot::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use crate::playback::Playback;

/// A session containing multiple players managed by a client.
pub struct Session {
    pub id: Uuid,
    pub playback: Playback,
    pub options: Mutex<SessionOptions>,
    pub cleanup: Mutex<Option<CancellationToken>>
}

pub struct SessionOptions {
    /// Whether if the session is resumable.
    pub enable_resume: bool,
    /// The time the session has to be resumed.
    pub timeout: Duration
}

impl Session {
    pub fn new(id: Uuid, user_id: NonZeroU64) -> Self {
        Self {
            id,
            playback: Playback::new(user_id),
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
