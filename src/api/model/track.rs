use serde::Serialize;
use songbird::input::AuxMetadata;

#[derive(Serialize, Debug)]
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