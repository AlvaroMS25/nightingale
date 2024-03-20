mod error;

use songbird::input::{AuxMetadata, Input};
pub use error::StringError;

pub mod yt;
mod youtube;

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

#[async_trait::async_trait]
pub trait SourcePlayer {
    async fn play_url(&self, _url: String) -> Result<Playable, StringError>;
}
