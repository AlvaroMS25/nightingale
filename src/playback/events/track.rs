use std::sync::Arc;
use songbird::{Event, EventContext, EventHandler};
use songbird::tracks::PlayMode;
use tokio::sync::RwLock;
use crate::api::model::gateway::{Outgoing, OutgoingEvent};
use crate::api::session::Session;
use crate::channel::Sender;
use crate::playback::metadata::TrackMetadata;

#[derive(Clone)]
pub struct TrackMetrics {
    #[allow(unused)]
    session: Arc<Session>,
    sender: Sender
}

impl TrackMetrics {
    pub async fn new(session: Arc<Session>) -> Self {
        let sender = session.playback.sharder.sender.clone();

        Self {
            session,
            sender
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for TrackMetrics {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let EventContext::Track([(state, handle), ..]) = ctx else { return None; };

        let map = handle.typemap().read().await;
        let metadata = map.get::<TrackMetadata>().unwrap(); // We insert it, so this cannot panic

        let event = match &state.playing {
            PlayMode::Play => OutgoingEvent::TrackStart(metadata.track()),
            PlayMode::Stop => OutgoingEvent::TrackEnd {
                stopped: true,
                track: metadata.track()
            },
            PlayMode::End => OutgoingEvent::TrackEnd {
                stopped: false,
                track: metadata.track()
            },
            PlayMode::Errored(error) => OutgoingEvent::TrackErrored {
                error: error.to_string(),
                track: metadata.track()
            },
            _ => return None,
        };

        let event = Outgoing::Event {
            guild_id: metadata.guild,
            event
        };

        drop(map);

        let _ = self.sender.send(event);

        None
    }
}
