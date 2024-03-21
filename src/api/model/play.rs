use serde::{Deserialize, Serialize};
use crate::api::model::track::Track;

/// Sources that can be used to play from.
#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum PlaySource {
    /// Provided by link, `yt-dlp` must support the provided source.
    Link {
        #[serde(default)]
        force_ytdlp: bool,
        link: String
    },
    Http {
        track: Track,
        link: String
    },
    /// Provided the whole track in bytes, ready to play without querying any more information.
    Bytes {
        track: Track,
        bytes: Vec<u8>
    }
}

impl PlaySource {
    pub fn into_inner(self) -> (String, Option<Track>) {
        match self {
            Self::Link { link, .. } => (link, None),
            _ => unreachable!()
        }
    }
}

/// Play options provided when requesting tracks.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlayOptions {
    /// Whether to pause the currently playing track and play the provided one,
    /// if this is set to `true`, the provided track will play at arrival, and the
    /// currently playing one will be resumed when it ends.
    pub force_play: bool,
    /// The track source.
    pub source: PlaySource
}
