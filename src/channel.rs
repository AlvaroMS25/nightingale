use std::borrow::Borrow;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use serde_json::{json, Value};
use songbird::error::{JoinError, JoinResult};
use songbird::id::{ChannelId, GuildId};
use songbird::shards::VoiceUpdate;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;

pub struct Sender<T>(mpsc::UnboundedSender<(u64, T)>);
pub struct Receiver<T>(mpsc::UnboundedReceiver<(u64, T)>);

impl<T> Sender<T> {
    fn send(&self, value: (u64, T)) -> Result<(), SendError<T>> {
        self.0.send(value)
            .map_err(|e| SendError(e.0.1))
    }
}

pub fn new<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = mpsc::unbounded_channel();

    (
        Sender(tx),
        Receiver(rx)
    )
}

impl<T> Sender<T> {
    pub fn for_shard(&self, shard_ud: u64) -> ForShard<T> {
        ForShard::new(self, shard_ud)
    }
}

pub struct ForShard<'a, T> {
    shard_id: u64,
    sender: &'a Sender<T>
}

impl<'a, T> ForShard<'a, T> {
    pub fn new(sender: &'a Sender<T>, shard_id: u64) -> Self {
        Self {
            shard_id,
            sender
        }
    }

    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.sender.send((self.shard_id, value))
    }

    pub fn into_owned(self) -> ForShardOwned<T> {
        ForShardOwned {
            shard_id: self.shard_id,
            sender: self.sender.clone()
        }
    }
}

pub struct ForShardOwned<T> {
    shard_id: u64,
    sender: Sender<T>
}

impl<T> ForShardOwned<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.sender.send((self.shard_id, value))
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Stream for Receiver<T> {
    type Item = (u64, T);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().0.poll_recv(cx)
    }
}

#[async_trait::async_trait]
impl VoiceUpdate for ForShardOwned<Value> {
    async fn update_voice_state(
        &self,
        guild_id: GuildId,
        channel_id: Option<ChannelId>,
        self_deaf: bool,
        self_mute: bool
    ) -> JoinResult<()> {
        let map = json!({
            "op": 4,
            "d": {
                "channel_id": channel_id.map(|c| c.0),
                "guild_id": guild_id.0,
                "self_deaf": self_deaf,
                "self_mute": self_mute,
            }
        });

        self.send(map)
            .map_err(|_| JoinError::NoSender)
    }
}
