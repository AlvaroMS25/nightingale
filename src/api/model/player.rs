use std::num::NonZeroU64;

use serde::Serialize;

use super::track::Track;

/// Serializable player object.
#[derive(Serialize, Debug)]
pub struct Player {
    pub guild_id: NonZeroU64,
    pub channel_id: Option<NonZeroU64>,
    pub paused: bool,
    pub volume: u8,
    pub currently_playing: Option<Track>,
    pub queue: Vec<Track>
}