use std::sync::Arc;
use dashmap::DashMap;
use songbird::id::{ChannelId, GuildId, UserId};
use songbird::Call;
use songbird::error::JoinResult;
use songbird::shards::{GenericSharder, Shard};
use tokio::sync::RwLock;
use twilight_model::gateway::event::Event;
use metrics::MetricsExt;
use crate::api::session::Session;
use crate::channel::Receiver;
use crate::playback::queue::Queue;
use crate::playback::sharder::Sharder;

mod mock;
mod sharder;
mod queue;
pub mod metadata;
mod resume;
pub mod metrics;

pub struct Playback {
    calls: DashMap<GuildId, Arc<RwLock<Call>>>,
    pub sharder: Sharder,
    pub receiver: Option<Receiver>,
    pub queue: Queue,
    pub user_id: UserId
}

impl Playback {
    pub fn new(shards: u64, user_id: impl Into<UserId>) -> Self {
        let (tx, rx) = crate::channel::new();
        let sharder = Sharder {
            shard_count: shards,
            sender: tx,
            map: Default::default()
        };

        Self {
            calls: DashMap::new(),
            sharder,
            receiver: Some(rx),
            queue: Default::default(),
            user_id: user_id.into()
        }
    }

    pub fn get_call(&self, guild: impl Into<GuildId>) -> Option<Arc<RwLock<Call>>> {
        self.calls.get(&guild.into())
            .map(|v| Arc::clone(v.value()))
    }

    pub async fn join<G, C>(
        &self, 
        guild: G, 
        channel_id: C, 
        s: Arc<RwLock<Session>>
    ) -> JoinResult<Arc<RwLock<Call>>> 
    where
        G: Into<GuildId>,
        C: Into<ChannelId>
    {
        let guild = guild.into();
        let channel_id = channel_id.into();
        let call = if self.calls.contains_key(&guild) {
            self.calls.get(&guild)
                .map(|v| Arc::clone(v.value()))
                .unwrap()
        } else {
            let shard = shard_id(guild.0.get(), self.sharder.shard_count);
            let mut c = Call::from_config(
                guild,
                Shard::Generic(self.sharder.get_shard(shard).expect("Failed to create Call, shard count incorrect")),
                self.user_id,
                Default::default()
            );
            c.register_metrics(s).await;
            let call = Arc::new(RwLock::new(c));
            call.write().await.add_global_event(
                songbird::CoreEvent::DriverConnect.into(),
                resume::ResumeOnMove::new(Arc::clone(&call))
            );

            self.calls.insert(guild, Arc::clone(&call));
            call
        };

        let stage_1 = {
            let mut handler = call.write().await;
            handler.join(channel_id).await
        };

        match stage_1 {
            Ok(chan) => chan.await.map(|()| call),
            Err(e) => Err(e),
        }
    }

    pub async fn leave(&self, g: impl Into<GuildId>) -> JoinResult<()> {
        let Some((_, call)) = self.calls.remove(&g.into()) else {
            return Ok(())
        };

        let mut write = call.write().await;
        write.leave().await
    }

    pub async fn process_event(&self, event: Event) {
        match event {
            Event::VoiceServerUpdate(su) => {
                let Some(c) = self.calls.get(&(su.guild_id.into())) else {
                    return;
                };

                if let Some(endpoint) = su.endpoint {
                    let mut write = c.write().await;
                    write.update_server(endpoint, su.token);
                }
            },
            Event::VoiceStateUpdate(su) => {
                if su.user_id.into_nonzero() != self.user_id.0 {
                    return;
                }

                let Some(c) = su.guild_id.and_then(|g| self.calls.get(&g.into())) else {
                    return;
                };

                let mut write = c.write().await;
                write.update_state(su.0.session_id, su.0.channel_id);
            },
            _ => {}
        }
    }
}

#[inline]
fn shard_id(guild_id: u64, shard_count: u64) -> u64 {
    (guild_id >> 22) % shard_count
}
