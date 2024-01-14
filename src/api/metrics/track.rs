use std::sync::Arc;
use songbird::{Event, EventContext, EventHandler};
use tokio::sync::RwLock;
use crate::api::session::Session;
use crate::channel::Sender;

pub struct TrackMetrics {
    session: Arc<RwLock<Session>>,
    sender: Sender
}

impl TrackMetrics {
    pub async fn new(session: Arc<RwLock<Session>>) -> Self {
        let sender = session.read().await.playback.sharder.sender.clone();

        Self {
            session,
            sender
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for TrackMetrics {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        todo!()
    }
}
