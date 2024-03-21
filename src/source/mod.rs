mod error;

use songbird::input::{AuxMetadata, Input};
pub use error::IntoResponseError;

pub mod yt;
mod youtube;
mod ytdlp;
mod http;

/// Helper that stores available search sources.
pub struct Search {
    pub youtube: yt::YoutubeSearch
}

impl Search {
    pub fn new() -> Self {
        Self {
            youtube: yt::YoutubeSearch::new()
        }
    }
}

pub struct Playable {
    input: Input,
    meta: AuxMetadata
}

/// Represents players that can play from an internet URL.
#[async_trait::async_trait]
pub trait SourcePlayer {
    async fn play_url(&self, _url: String) -> Result<Playable, IntoResponseError>;
}
