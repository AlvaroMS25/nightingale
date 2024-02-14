use serde_json::Value;
use crate::api::model::track::Track;
use crate::api::model;
use crate::api::model::voice::{UpdateVoiceServer, UpdateVoiceState, VoiceEvent};

/// Possible incoming payloads from clients via websocket.
#[derive(serde::Deserialize, Debug)]
#[serde(tag = "op", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Incoming {
    UpdateVoiceServer(UpdateVoiceServer),
    UpdateVoiceState(UpdateVoiceState),
}

impl Incoming {
    pub fn is_voice_event(&self) -> bool {
        matches!(self, Incoming::UpdateVoiceState(_)) || matches!(self, Incoming::UpdateVoiceServer(_))
    }
}

impl Into<VoiceEvent> for Incoming {
    fn into(self) -> VoiceEvent {
        match self {
            Self::UpdateVoiceServer(update) => VoiceEvent::UpdateVoiceServer(update),
            Self::UpdateVoiceState(state) => VoiceEvent::UpdateVoiceState(state),
        }
    }
}

/// Events sent via websocket to clients.
#[non_exhaustive]
#[derive(serde::Serialize, Debug)]
#[serde(tag = "op", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Outgoing {
    /// The provided payload should be forwarded to discord's gateway via the specified shard.
    Forward {
        /// The shard that should forward the payload.
        shard: u64,
        /// The payload to forward.
        payload: Value
    },
    /// Event sent on client websocket connection.
    Ready(model::ready::Ready),
    /// Track related events.
    Event {
        /// Guild id corresponding to the event.
        guild_id: u64,
        /// The event itself.
        event: OutgoingEvent
    },
    /// Gateway connection related events.
    UpdateState(super::connection::UpdateState)
}

/// Track related events.
#[non_exhaustive]
#[derive(serde::Serialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum OutgoingEvent {
    /// A track started its playback.
    TrackStart(Track),
    /// A track ended, either naturally or manually
    TrackEnd {
        /// Whether the track was stopped manually.
        stopped: bool,
        /// The track itself.
        track: Track
    },
    /// A track had an error on playback.
    TrackErrored {
        /// The error message.
        error: String,
        /// The track itself.
        track: Track
    }
}

