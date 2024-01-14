use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Ready {
    pub session: Uuid,
    pub resumed: bool
}
