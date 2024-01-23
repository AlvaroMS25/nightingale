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
