use serde_json::Value;
use crate::api::model::track::Track;
use crate::api::model;
use crate::api::model::voice::{UpdateVoiceServer, UpdateVoiceState, VoiceEvent};

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

#[non_exhaustive]
#[derive(serde::Serialize, Debug)]
#[serde(tag = "op", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Outgoing {
    Forward {
        shard: u64,
        payload: Value
    },
    Ready(model::ready::Ready),
    Event {
        guild_id: u64,
        event: OutgoingEvent
    },
    UpdateState(super::connection::UpdateState)
}

#[non_exhaustive]
#[derive(serde::Serialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum OutgoingEvent {
    TrackStart(Track),
    TrackEnd {
        stopped: bool,
        track: Track
    },
    TrackErrored {
        error: String,
        track: Track
    }
}

