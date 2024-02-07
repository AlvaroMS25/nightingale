use std::num::NonZeroU64;

use serde::Serialize;

use super::track::Track;

#[derive(Serialize)]
pub struct Player {
    guild_id: NonZeroU64,
    channel_id: NonZeroU64,
    paused: bool,
    volume: u8,
    currently_playing: Option<Track>,
    queue: Vec<Track>
}