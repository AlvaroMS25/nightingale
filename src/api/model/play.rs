use bytes::Bytes;
use serde::{Deserialize, Serialize};
use crate::api::model::track::Track;

/// Sources that can be used to play from.
#[derive(Deserialize, Serialize, Clone)]
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
        #[serde(default)]
        track: Option<Track>,
        link: String
    },
    /// Provided the whole track in bytes, ready to play without querying any more information.
    Bytes {
        #[serde(default)]
        track: Option<Track>,
        // Bytes is cheaply cloneable because it is only a pointer clone, so if we want to keep a
        // copy to repeat the queue, this is a great way of avoiding extra allocations
        bytes: Bytes
    }
}

impl PlaySource {
    pub fn is_link(&self) -> bool {
        matches!(self, Self::Link {..})
    }

    pub fn url(&self) -> String {
        match self {
            Self::Link {link, ..} => link.clone(),
            Self::Http {link, ..} => link.clone(),
            Self::Bytes {..} => unreachable!()
        }
    }

    pub fn track(&mut self) -> Option<Track> {
        match self {
            Self::Link {..} => None,
            Self::Http {track, .. } => track.take(),
            Self::Bytes {track, .. } => track.take()
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
