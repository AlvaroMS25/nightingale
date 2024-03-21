use std::time::Duration;
use rusty_ytdl::search::{Playlist, Video};
use rusty_ytdl::{VideoDetails, VideoFormat};
use serde::Serialize;
use songbird::input::AuxMetadata;
use crate::ext::VecExt;
use crate::source::IntoResponseError;

#[derive(Serialize)]
pub struct YoutubeTrack {
    /// Title of the track.
    pub title: String,
    /// Author of the track if available.
    pub author: Option<String>,
    /// Length of the track in milliseconds.
    pub length: u64,
    /// Id of the video.
    pub video_id: String,
    /// The url of the video.
    pub url: String,
    /// The thumbnail of the video.
    pub thumbnail: Option<String>
}

#[derive(Serialize)]
pub struct YoutubePlaylist {
    /// Name of the playlist.
    pub name: String,
    pub id: String,
    pub url: String,
    pub channel: String,
    pub thumbnail: Option<String>,
    /// Tracks of the playlist.
    pub tracks: Vec<YoutubeTrack>
}

impl From<Video> for YoutubeTrack {
    fn from(mut video: Video) -> Self {
        YoutubeTrack {
            title: video.title,
            author: Some(video.channel.name),
            length: video.duration * 1000,
            video_id: video.id,
            url: video.url,
            thumbnail: video.thumbnails.remove_optional(0).map(|t| t.url)
        }
    }
}

impl From<Playlist> for YoutubePlaylist {
    fn from(mut playlist: Playlist) -> Self {
        YoutubePlaylist {
            name: playlist.name,
            id: playlist.id,
            url: playlist.url,
            channel: playlist.channel.url,
            thumbnail: playlist.thumbnails.remove_optional(0).map(|p| p.url),
            tracks: playlist.videos.into_iter().map(Into::into).collect()
        }
    }
}

pub(super) struct WrapInfo(pub VideoDetails, pub VideoFormat);

impl TryFrom<WrapInfo> for AuxMetadata {
    type Error = IntoResponseError;

    fn try_from(value: WrapInfo) -> Result<Self, Self::Error> {
        use std::mem::take;

        let mut details = value.0;
        let format = value.1;

        Ok(AuxMetadata {
            track: None,
            artist: details.author.as_mut().map(|a| take(&mut a.name)),
            album: None,
            date: Some(details.publish_date),
            channels: format.audio_channels,
            channel: details.author.as_mut().map(|a| take(&mut a.channel_url)),
            start_time: None,
            duration: Some(Duration::from_secs(details.length_seconds.parse()?)),
            sample_rate: format.audio_sample_rate.as_ref().map(|s| s.parse()).transpose()?,
            source_url: Some(details.video_url),
            title: Some(details.title),
            thumbnail: details.thumbnails.remove_optional(0).map(|t| t.url)
        })
    }
}
