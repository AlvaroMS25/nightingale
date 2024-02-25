use std::time::Duration;
use serde::{Deserialize, Serialize};
use songbird::input::AuxMetadata;

/// Serializable songbird track.
#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    pub track: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub channel: Option<String>,
    pub duration: Option<u128>,
    pub source_url: Option<String>,
    pub title: Option<String>,
    pub thumbnail: Option<String>,
}

impl From<&AuxMetadata> for Track {
    fn from(value: &AuxMetadata) -> Self {
        Self {
            track: value.track.clone(),
            artist: value.artist.clone(),
            album: value.album.clone(),
            channel: value.channel.clone(),
            duration: value.duration.map(|d| d.as_millis()),
            source_url: value.source_url.clone(),
            title: value.title.clone(),
            thumbnail: value.thumbnail.clone()
        }
    }
}

impl From<Track> for AuxMetadata {
    fn from(value: Track) -> Self {
        AuxMetadata {
            track: value.track,
            artist: value.artist,
            album: value.album,
            channel: value.channel,
            duration: value.duration.map(|d| Duration::from_millis(d as _)),
            source_url: value.source_url,
            title: value.title,
            thumbnail: value.thumbnail,
            ..Default::default()
        }
    }
}
