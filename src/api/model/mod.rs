pub mod resume;
pub mod play;

use twilight_model::gateway::payload::incoming::{VoiceServerUpdate, VoiceStateUpdate};
use twilight_model::voice::VoiceState;
use twilight_model::gateway::event::Event;

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
