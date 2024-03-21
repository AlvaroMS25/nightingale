mod error;

use reqwest::Client;
use songbird::input::{AuxMetadata, Input};
pub use error::IntoResponseError;
use crate::source::http::HttpSource;
use crate::source::youtube::Youtube;
use crate::source::ytdlp::Ytdlp;

pub mod youtube;
pub mod ytdlp;
pub mod http;

pub struct Sources {
    pub youtube: Youtube,
    pub yt_dlp: Ytdlp,
    pub http: HttpSource
}

impl Sources {
    pub fn new(http: Client) -> Self {
        Self {
            youtube: Youtube::new(http.clone()),
            yt_dlp: Ytdlp::new(http.clone()),
            http: HttpSource::new(http)
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
