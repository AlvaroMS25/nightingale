use std::sync::Arc;
use songbird::{Event, EventContext, EventHandler};
use crate::api::model::{connection::{ConnectionData, DisconnectData, UpdateState}, gateway::Outgoing};
use crate::api::session::Session;
use crate::channel::Sender;

/// Event listener for driver events.
#[derive(Clone)]
pub struct DriverMetrics {
    #[allow(unused)]
    session: Arc<Session>,
    sender: Sender
}

impl DriverMetrics {
    pub async fn new(session: Arc<Session>) -> Self {
        let sender = session.playback.sharder.sender.clone();

        Self {
            session,
            sender
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for DriverMetrics {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let state = match ctx {
            EventContext::DriverConnect(d) => UpdateState::ConnectGateway(ConnectionData {
                channel_id: d.channel_id.map(|c| c.0),
                guild_id: d.guild_id.0,
                session_id: d.session_id.to_string(),
                server: d.server.to_string(),
                ssrc: d.ssrc
            }),
            EventContext::DriverDisconnect(d) => UpdateState::DisconnectGateway(DisconnectData {
                channel_id: d.channel_id.map(|c| c.0),
                guild_id: d.guild_id.0,
                session_id: d.session_id.to_string()
            }),
            EventContext::DriverReconnect(d) => UpdateState::ReconnectGateway(ConnectionData {
                channel_id: d.channel_id.map(|c| c.0),
                guild_id: d.guild_id.0,
                session_id: d.session_id.to_string(),
                server: d.server.to_string(),
                ssrc: d.ssrc
            }),
            _ => return None,
        };

        let _ = self.sender.send(Outgoing::UpdateState(state));

        None
    }
}
