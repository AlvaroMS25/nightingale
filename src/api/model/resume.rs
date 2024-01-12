use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct ResumeSession {
    pub session: Uuid,
    pub resumed: bool
}

#[derive(Deserialize)]
pub struct UpdateSession {
    pub enable_resume: bool,
    pub timeout: usize
}
