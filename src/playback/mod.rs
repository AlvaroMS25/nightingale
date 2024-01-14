use std::sync::Arc;
use songbird::id::UserId;
use songbird::Songbird;
use crate::channel::Receiver;
use crate::playback::queue::Queue;
use crate::playback::sharder::Sharder;

mod sharder;
mod queue;
pub mod metadata;

pub struct Playback {
    pub songbird: Songbird,
    pub sharder: Arc<Sharder>,
    pub receiver: Option<Receiver>,
    pub queue: Queue
}

impl Playback {
    pub fn new(shards: u64, user_id: impl Into<UserId>) -> Self {
        let (tx, rx) = crate::channel::new();
        let sharder = Arc::new(Sharder {
            shard_count: shards,
            sender: tx,
            map: Default::default()
        });

        Self {
            songbird: Songbird::custom_from_config(Arc::clone(&sharder) as Arc<_>, user_id, Default::default()),
            sharder,
            receiver: Some(rx),
            queue: Default::default()
        }
    }
}
