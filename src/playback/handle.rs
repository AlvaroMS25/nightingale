use bytes::Bytes;
use songbird::tracks::TrackHandle;
use crate::api::model::play::PlaySource;
use crate::playback::metadata::TrackMetadata;

pub enum MinimalSource {
    Link {
        force_ytdlp: bool,
        link: String
    },
    Http(String),
    Bytes(Bytes)
}

impl From<PlaySource> for MinimalSource {
    fn from(value: PlaySource) -> Self {
        match value {
            PlaySource::Bytes { bytes, .. } => MinimalSource::Bytes(bytes),
            PlaySource::Http { link, .. } => MinimalSource::Http(link),
            PlaySource::Link { force_ytdlp, link } => MinimalSource::Link {
                force_ytdlp,
                link
            }
        }
    }
}

pub struct HandleWithSource {
    pub handle: TrackHandle,
    pub source: MinimalSource
}

impl HandleWithSource {
    pub fn new(handle: TrackHandle, source: MinimalSource) -> Self {
        Self {
            handle,
            source
        }
    }

    pub async fn full_source(&self) -> PlaySource {
        let read = self.handle.typemap().read().await;

        let track = read.get::<TrackMetadata>().unwrap().track();

        match &self.source {
            MinimalSource::Link {force_ytdlp, link} => PlaySource::Link {
                force_ytdlp: *force_ytdlp,
                link: link.clone()
            },
            MinimalSource::Http(link) => PlaySource::Http {
                link: link.clone(),
                track: Some(track)
            },
            MinimalSource::Bytes(bytes) => PlaySource::Bytes {
                bytes: bytes.clone(),
                track: Some(track)
            },
        }
    }
}
