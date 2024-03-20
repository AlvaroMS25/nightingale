mod error;

use songbird::input::AuxMetadata;
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

pub trait SourcePlayer {
    async fn play_url(_url: String) -> Result<AuxMetadata, StringError>;
}
