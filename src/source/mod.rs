use bytes::Bytes;
use reqwest::Client;
use songbird::input::{AuxMetadata, Input};
use crate::api::error::IntoResponseError;
use crate::api::model::play::PlaySource;
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

    pub fn source_for(&self, source: &PlaySource) -> &dyn SourcePlayer {
        match source {
            PlaySource::Link { force_ytdlp, link } => {
                if *force_ytdlp || !self.youtube.can_play(link.as_str()) {
                    &self.yt_dlp
                } else {
                    &self.youtube
                }
            },
            PlaySource::Http {..} => &self.http,
            _ => unreachable!()
        }
    }

    pub async fn playable_for(&self, source: &mut PlaySource) -> Result<Playable, IntoResponseError> {
        Ok(match source {
            PlaySource::Bytes {track, bytes} => Playable {
                input: Input::from(<Bytes as Into<Input>>::into(bytes.clone())),
                meta: track.clone().map(|t| t.into()).unwrap_or_default()
            },
            other => {
                let source = self.source_for(other);
                let url = other.url();

                let mut playable = source.play_url(url).await?;

                if let Some(t) = other.track() {
                    playable.meta = t.into();
                }

                playable
            }
        })
    }
}

pub struct Playable {
    pub input: Input,
    pub meta: AuxMetadata
}

/// Represents players that can play from an internet URL.
#[async_trait::async_trait]
pub trait SourcePlayer {
    async fn play_url(&self, _url: String) -> Result<Playable, IntoResponseError>;
}
