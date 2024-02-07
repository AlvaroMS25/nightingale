use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::Mutex;
use songbird::id::{ChannelId, GuildId, UserId};
use songbird::Call;
use songbird::error::JoinResult;
use songbird::shards::{GenericSharder, Shard};
use tokio::sync::Mutex as AsyncMutex;
use events::EventsExt;
use crate::api::model::voice::VoiceEvent;
use crate::api::session::Session;
use crate::channel::Receiver;
use crate::playback::player::Player;
use crate::playback::sharder::Sharder;

mod sharder;
pub mod metadata;
pub mod events;
pub mod player;

pub struct Playback {
    pub players: DashMap<GuildId, Arc<AsyncMutex<Player>>>,
    pub sharder: Sharder,
    pub receiver: Mutex<Option<Receiver>>,
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
            players: DashMap::new(),
            sharder,
            receiver: Mutex::new(Some(rx)),
            user_id: user_id.into()
        }
    }

    pub fn get_player(&self, guild: impl Into<GuildId>) -> Option<Arc<AsyncMutex<Player>>> {
        self.players.get(&guild.into())
            .map(|v| Arc::clone(v.value()))
    }

    pub async fn join<G, C>(
        &self, 
        guild: G, 
        channel_id: C, 
        s: Arc<Session>
    ) -> JoinResult<Arc<AsyncMutex<Player>>>
    where
        G: Into<GuildId>,
        C: Into<ChannelId>
    {
        let guild = guild.into();
        let channel_id = channel_id.into();
        let player = if self.players.contains_key(&guild) {
            self.players.get(&guild)
                .map(|v| Arc::clone(v.value()))
                .unwrap()
        } else {
            let shard = shard_id(guild.0.get(), self.sharder.shard_count);
            let mut call = Call::from_config(
                guild,
                Shard::Generic(self.sharder.get_shard(shard).expect("Failed to create Call, shard count incorrect")),
                self.user_id,
                Default::default()
            );
            call.register_events(s).await;

            let player = Player::new(call).await;

            self.players.insert(guild, Arc::clone(&player));
            player
        };

        let stage_1 = {
            let mut handler = player.lock().await;
            handler.call.join(channel_id).await
        };

        match stage_1 {
            Ok(chan) => chan.await.map(|()| player),
            Err(e) => Err(e),
        }
    }

    pub async fn leave(&self, g: impl Into<GuildId>) -> JoinResult<()> {
        let Some((_, call)) = self.players.remove(&g.into()) else {
            return Ok(())
        };

        let mut write = call.lock().await;
        write.destroy().await
    }

    pub async fn process_event(&self, event: VoiceEvent) {
        match event {
            VoiceEvent::UpdateVoiceServer(su) => {
                let Some(c) = self.players.get(&(su.guild_id.0.into())) else {
                    return;
                };

                if let Some(endpoint) = su.endpoint {
                    let mut write = c.lock().await;
                    write.call.update_server(endpoint, su.token);
                }
            },
            VoiceEvent::UpdateVoiceState(su) => {
                if su.user_id.0 != self.user_id.0 {
                    return;
                }

                let Some(c) = su.guild_id.and_then(|g| self.players.get(&g.0.into())) else {
                    return;
                };

                let mut write = c.lock().await;
                write.call.update_state(su.session_id, su.channel_id.map(|i| i.0));
            },
        }
    }
}

#[inline]
fn shard_id(guild_id: u64, shard_count: u64) -> u64 {
    (guild_id >> 22) % shard_count
}
