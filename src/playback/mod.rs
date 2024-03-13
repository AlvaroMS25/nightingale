use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::Mutex;
use songbird::id::{ChannelId, GuildId, UserId};
use songbird::{Call, Driver};
use songbird::error::JoinResult;
use songbird::shards::{GenericSharder, Shard};
use tokio::sync::Mutex as AsyncMutex;
use tracing::debug;
use events::EventsExt;
use crate::api::model::voice::VoiceEvent;
use crate::api::session::Session;
use crate::channel::{Receiver, Sender};
use crate::playback::player::handler::PlaybackHandler;
use crate::playback::player::Player;
use crate::playback::sharder::Sharder;

pub mod sharder;
pub mod metadata;
pub mod events;
pub mod player;

pub struct Playback {
    pub players: DashMap<GuildId, Arc<AsyncMutex<Player>>>,
    pub receiver: Mutex<Option<Receiver>>,
    pub sender: Sender,
    pub user_id: UserId
}

impl Playback {
    pub fn new(user_id: impl Into<UserId>) -> Self {
        let (tx, rx) = crate::channel::new();

        Self {
            players: DashMap::new(),
            sender: tx,
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
    ) -> Arc<AsyncMutex<Player>>
    where
        G: Into<GuildId>,
        C: Into<ChannelId>
    {
        let guild = guild.into();
        let channel_id = channel_id.into();
        if self.players.contains_key(&guild) {
            self.players.get(&guild)
                .map(|v| Arc::clone(v.value()))
                .unwrap()
        } else {
            let mut player = Player::new(guild).await;
            player.register_events(s).await;

            let player = Arc::new(AsyncMutex::new(player));
            PlaybackHandler::register(Arc::clone(&player)).await;

            self.players.insert(guild, Arc::clone(&player));
            player
        }
    }

    pub async fn leave(&self, g: impl Into<GuildId>) -> JoinResult<()> {
        let Some((_, call)) = self.players.remove(&g.into()) else {
            return Ok(())
        };

        let mut write = call.lock().await;
        write.destroy().await
    }

    pub async fn destroy(&self) {
        let keys = self.players.iter()
            .map(|i| *i.key())
            .collect::<Vec<_>>();

        for id in keys {
            let _ = self.leave(id).await;
        }
    }
}
