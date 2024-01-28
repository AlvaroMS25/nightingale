use std::num::NonZeroU64;
use serde::Deserialize;

pub enum VoiceEvent {
    UpdateVoiceServer(UpdateVoiceServer),
    UpdateVoiceState(UpdateVoiceState)
}

#[derive(Deserialize, Debug)]
pub struct UpdateVoiceServer {
    pub endpoint: Option<String>,
    pub guild_id: NonZeroU64,
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateVoiceState {
    pub guild_id: Option<NonZeroU64>,
    pub user_id: NonZeroU64,
    pub session_id: String,
    pub channel_id: Option<NonZeroU64>
}
