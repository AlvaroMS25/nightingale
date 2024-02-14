use std::num::NonZeroU64;

/// Possible `update_state` payloads.
#[non_exhaustive]
#[derive(serde::Serialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum UpdateState {
    ConnectGateway(ConnectionData),
    ReconnectGateway(ConnectionData),
    DisconnectGateway(DisconnectData)
}

/// Information about a connection to a voice channel.
#[derive(serde::Serialize, Debug)]
pub struct ConnectionData {
    /// Channel id corresponding to the connection, if available.
    pub channel_id: Option<NonZeroU64>,
    /// Guild id the connection is on.
    pub guild_id: NonZeroU64,
    /// The session id of the connection.
    pub session_id: String,
    /// The address of the connection server.
    pub server: String,
    /// Ssrc of the connection.
    pub ssrc: u32
}

/// Information about a disconnection from a channel.
#[derive(serde::Serialize, Debug)]
pub struct DisconnectData {
    /// Channel id disconnected from, if available.
    pub channel_id: Option<NonZeroU64>,
    /// Guild id disconnected from.
    pub guild_id: NonZeroU64,
    /// The session if of the connectio.
    pub session_id: String
}
