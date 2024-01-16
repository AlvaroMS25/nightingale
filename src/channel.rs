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
use crate::api::model::gateway::Outgoing;

pub struct Sender(mpsc::UnboundedSender<Outgoing>);
pub struct Receiver(mpsc::UnboundedReceiver<Outgoing>);

impl Sender {
    pub fn send(&self, value: Outgoing) -> Result<(), SendError<Outgoing>> {
        self.0.send(value)
    }
}

pub fn new<T>() -> (Sender, Receiver) {
    let (tx, rx) = mpsc::unbounded_channel();

    (
        Sender(tx),
        Receiver(rx)
    )
}

impl Sender {
    pub fn for_shard(&self, shard_ud: u64) -> ForShard {
        ForShard::new(self, shard_ud)
    }
}

pub struct ForShard<'a> {
    shard_id: u64,
    sender: &'a Sender
}

impl<'a> ForShard<'a> {
    pub fn new(sender: &'a Sender, shard_id: u64) -> Self {
        Self {
            shard_id,
            sender
        }
    }

    pub fn send(&self, value: Value) -> Result<(), SendError<Value>> {
        self.sender.send(Outgoing::Forward {
            shard: self.shard_id,
            payload: value
        }).map_err(|e| {
            let Outgoing::Forward { payload, ..} = e.0 else { unreachable!() };
            SendError(payload)
        })
    }

    pub fn into_owned(self) -> ForShardOwned {
        ForShardOwned {
            shard_id: self.shard_id,
            sender: self.sender.clone()
        }
    }
}

pub struct ForShardOwned {
    shard_id: u64,
    sender: Sender
}

impl ForShardOwned {
    pub fn send(&self, value: Value) -> Result<(), SendError<Value>> {
        ForShard {
            shard_id: self.shard_id,
            sender: &self.sender
        }.send(value)
    }
}

impl Clone for Sender {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Stream for Receiver {
    type Item = Outgoing;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().0.poll_recv(cx)
    }
}

#[async_trait::async_trait]
impl VoiceUpdate for ForShardOwned {
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
