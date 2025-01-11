use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::Mutex;
use songbird::Config;
use songbird::driver::DisposalThread;
use songbird::id::{GuildId, UserId};
use songbird::error::ConnectionError;
use tracing::info;
use crate::api::session::Session;
use crate::channel::{Receiver, Sender};
use crate::mutex::TicketedMutex;
use crate::playback::player::handler::PlayerHandler;
use crate::playback::player::Player;
use crate::ptr::SharedPtr;
use crate::source::Sources;

pub mod metadata;
pub mod events;
pub mod filters;
pub mod player;
mod handle;

pub struct Playback {
    pub players: DashMap<GuildId, Arc<TicketedMutex<Player>>>,
    pub receiver: Mutex<Option<Receiver>>,
    pub sender: Sender,
    pub user_id: UserId,
    pub disposer: DisposalThread,
    pub sources: SharedPtr<Sources>
}

impl Playback {
    pub fn new(user_id: impl Into<UserId>, sources: SharedPtr<Sources>) -> Self {
        let (tx, rx) = crate::channel::new();

        Self {
            players: DashMap::new(),
            sender: tx,
            receiver: Mutex::new(Some(rx)),
            user_id: user_id.into(),
            disposer: DisposalThread::run(),
            sources
        }
    }

    pub fn get_player(&self, guild: impl Into<GuildId>) -> Option<Arc<TicketedMutex<Player>>> {
        self.players.get(&guild.into())
            .map(|v| Arc::clone(v.value()))
    }

    pub async fn get_or_create<G>(
        &self, 
        guild: G,
        s: Arc<Session>
    ) -> Arc<TicketedMutex<Player>>
    where
        G: Into<GuildId>,
    {
        let guild = guild.into();
        if self.players.contains_key(&guild) {
            self.players.get(&guild)
                .map(|v| Arc::clone(v.value()))
                .unwrap()
        } else {
            let mut player = Player::new(
                guild,
                self.sources.clone(),
                Config::default()
                    .disposer(self.disposer.clone()),
                self.sender.clone()
            );
            player.register_events(s).await;

            info!("Created player for guild {guild}");

            let player = Arc::new(TicketedMutex::new(player));
            PlayerHandler::register(Arc::clone(&player));

            self.players.insert(guild, Arc::clone(&player));
            player
        }
    }

    pub async fn destroy_player(&self, g: impl Into<GuildId>) -> Result<(), ConnectionError> {
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
            let _ = self.destroy_player(id).await;
        }
    }
}
