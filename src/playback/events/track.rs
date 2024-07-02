use std::sync::Arc;
use songbird::{Event, EventContext, EventHandler};
use songbird::tracks::PlayMode;
use crate::api::model::gateway::{Outgoing, OutgoingEvent};
use crate::api::session::Session;
use crate::channel::Sender;
use crate::playback::metadata::TrackMetadata;

/// Track related events listener.
#[derive(Clone)]
pub struct TrackEvents {
    #[allow(unused)]
    session: Arc<Session>,
    sender: Sender
}

impl TrackEvents {
    pub fn new(session: Arc<Session>) -> Self {
        let sender = session.playback.sender.clone();

        Self {
            session,
            sender
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for TrackEvents {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let EventContext::Track([(state, handle), ..]) = ctx else { return None; };

        let metadata = handle.data::<TrackMetadata>();

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

        let _ = self.sender.send(event);

        None
    }
}
