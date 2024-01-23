use serde_json::Value;
use twilight_model::gateway::payload::incoming::{VoiceServerUpdate, VoiceStateUpdate};
use twilight_model::voice::VoiceState;
use twilight_model::gateway::event::Event;
use crate::api::model::track::Track;
use crate::api::model;

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "op", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Incoming {
    UpdateVoiceServer(VoiceServerUpdate),
    UpdateVoiceState(VoiceState),
}

impl Incoming {
    pub fn is_voice_event(&self) -> bool {
        matches!(self, Incoming::UpdateVoiceState(_)) || matches!(self, Incoming::UpdateVoiceServer(_))
    }
}

impl Into<Event> for Incoming {
    fn into(self) -> Event {
        match self {
            Self::UpdateVoiceServer(update) => Event::VoiceServerUpdate(update),
            Self::UpdateVoiceState(state) => Event::VoiceStateUpdate(Box::new(VoiceStateUpdate(state))),
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
        guild: u64,
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

