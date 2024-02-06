use std::num::NonZeroU64;

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
    pub channel_id: Option<NonZeroU64>,
    pub guild_id: NonZeroU64,
    pub session_id: String,
    pub server: String,
    pub ssrc: u32
}

#[derive(serde::Serialize, Debug)]
pub struct DisconnectData {
    pub channel_id: Option<NonZeroU64>,
    pub guild_id: NonZeroU64,
    pub session_id: String
}
