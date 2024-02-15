use std::sync::Arc;
use dashmap::DashMap;
use songbird::shards::{GenericSharder, VoiceUpdate};
use crate::channel::{ForShardOwned, Sender};

/// Sharder that manages shard senders provided to [`Call`](songbird::Call)s
pub struct Sharder {
    pub shard_count: u64,
    pub sender: Sender,
    pub map: DashMap<u64, Arc<ForShardOwned>>
}

impl GenericSharder for Sharder {
    fn get_shard(&self, shard_id: u64) -> Option<Arc<dyn VoiceUpdate + Send + Sync>> {
        if shard_id > self.shard_count {
            None
        } else {
            match self.map.get(&shard_id) {
                Some(sender) => Some(Arc::clone(sender.value()) as Arc<_>),
                None => {
                    let sender = Arc::new(self.sender.for_shard(shard_id).into_owned());
                    self.map.insert(shard_id, Arc::clone(&sender));
                    Some(sender as Arc<_>)
                }
            }
        }
    }
}
