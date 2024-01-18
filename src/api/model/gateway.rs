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
            _ => unreachable!()
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
    UpdateState(UpdateState)
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

#[non_exhaustive]
#[derive(serde::Serialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum UpdateState {
    ConnectGateway(ConnectionData),
    ReconnectGateway(ConnectionData),
    DisconnectGateway(DisconnectData)
}

#[derive(serde::Serialize, Debug)]
pub struct ConnectionData {
    pub channel: Option<u64>,
    pub guild: u64,
    pub session: String,
    pub server: String,
    pub ssrc: u32
}

#[derive(serde::Serialize, Debug)]
pub struct DisconnectData {
    pub channel: Option<u64>,
    pub guild: u64,
    pub session: String
}
