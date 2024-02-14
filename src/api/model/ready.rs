use serde::Serialize;
use uuid::Uuid;

/// Ready object sent on connection establishment.
#[derive(Serialize, Debug)]
pub struct Ready {
    /// The session id of the connection.
    pub session: Uuid,
    /// Whether if the connection has been resumed.
    pub resumed: bool
}
