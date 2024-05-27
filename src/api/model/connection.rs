use std::num::NonZeroU64;
use songbird::ConnectionInfo;
use crate::api::serde::nz::NzU64;

/// Possible `update_state` payloads.
#[non_exhaustive]
#[allow(clippy::enum_variant_names)]
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

/// Connection information used to connect to a voice channel
#[derive(serde::Deserialize, Debug)]
pub struct DeserializableConnectionInfo {
    /// Channel id to connect to.
    pub channel_id: Option<NzU64>,
    /// Endpoint to connect to.
    pub endpoint: String,
    /// Session id of the connection.
    pub session_id: String,
    /// Token of the connection.
    pub token: String
}

impl DeserializableConnectionInfo {
    pub fn into_songbird(self, user_id: NonZeroU64, guild_id: NonZeroU64) -> ConnectionInfo {
        ConnectionInfo {
            channel_id: self.channel_id.map(Into::into),
            endpoint: self.endpoint,
            guild_id: guild_id.into(),
            session_id: self.session_id,
            token: self.token,
            user_id: user_id.into()
        }
    }
}
