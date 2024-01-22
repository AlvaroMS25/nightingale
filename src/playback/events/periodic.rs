use std::sync::Arc;
use songbird::{Event, EventContext, EventHandler};
use tokio::sync::RwLock;
use crate::api::session::Session;
use crate::channel::Sender;

pub struct PeriodicMetrics {
    session: Arc<RwLock<Session>>,
    sender: Sender
}

impl PeriodicMetrics {
    pub async fn new(session: Arc<RwLock<Session>>) -> Self {
        let sender = session.read().await.playback.sharder.sender.clone();

        Self {
            session,
            sender
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for PeriodicMetrics {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        None
    }
}
