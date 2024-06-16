use serde::Serialize;
use uuid::Uuid;
use crate::api::model::player::Player;

/// Ready object sent on connection establishment.
#[derive(Serialize, Debug)]
pub struct Ready {
    /// The session id of the connection.
    pub session: Uuid,
    /// Whether if the connection has been resumed.
    pub resumed: bool,
    /// Players of the session, only sent if the session is being resumed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<Vec<Player>>
}
