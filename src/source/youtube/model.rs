use rusty_ytdl::search::{Playlist, Video};
use serde::Serialize;
use crate::ext::VecExt;

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
    fn from(video: Video) -> Self {
        YoutubeTrack {
            title: video.title,
            author: Some(video.channel.name),
            length: video.duration * 1000,
            video_id: video.id,
            url: video.url,
            thumbnail: video.thumbnails.get(0).map(|t| t.url.clone())
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
            thumbnail: playlist.thumbnails.remove_optional(0).map(|p| p.url.clone()),
            tracks: playlist.videos.into_iter().map(Into::into).collect()
        }
    }
}
