use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Debug)]
pub struct Ready {
    pub session: Uuid,
    pub resumed: bool
}
