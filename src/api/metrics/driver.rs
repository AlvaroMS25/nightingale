use std::sync::Arc;
use tokio::sync::RwLock;
use crate::api::session::Session;
use crate::channel::Sender;

pub struct DriverMetrics {
    session: Arc<RwLock<Session>>,
    sender: Sender
}

impl DriverMetrics {
    pub async fn new(session: Arc<RwLock<Session>>) -> Self {
        let sender = session.read().await.playback.sharder.sender.clone();

        Self {
            session,
            sender
        }
    }
}