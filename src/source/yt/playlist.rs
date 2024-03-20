use std::time::Duration;
use serde::Serialize;
use serde_json::Value;
use crate::source::yt::{get_thumbnail, WATCH_URL_PREFIX};
use crate::source::yt::track::YoutubeTrack;

#[derive(Serialize)]
pub struct YoutubePlaylist {
    /// Name of the playlist.
    pub name: String,
    /// Tracks of the playlist.
    pub tracks: Vec<YoutubeTrack>
}

impl YoutubePlaylist {
    pub fn parse(mut object: Value, out: &mut Vec<YoutubeTrack>) -> Option<String> {
        let Some(contents_value) = object.get_mut("contents") else { return None; };

        let Some(contents_array) = contents_value.as_array_mut() else { return None; };

        for value in contents_array.drain(..) {
            let item = value.get("playlistVideoRenderer").unwrap_or(&Value::Null);
            let short_by_line = item.get("shortBylineText").unwrap_or(&Value::Null);

            if !item.is_null() && !short_by_line.is_null() {
                if let Some(track) = Self::parse_playlist_track(item, short_by_line) {
                    out.push(track);
                }
            }
        }

        object.get("continuations")?.as_array()?
            .get(0)?
            .get("nextContinuationData")?
            .get("continuation")?
            .as_str()
            .map(ToString::to_string)
    }

    pub fn parse_playlist_track(item: &Value, short_by_line: &Value) -> Option<YoutubeTrack> {
        let video_id = item.get("videoId")?.as_str()?.to_string();
        let author = short_by_line.get("runs")?.as_array()?
            .get(0)?.get("text")
            .map(|author| author.as_str().map(ToString::to_string))
            .flatten();
        let length = Duration::from_secs(item.get("lengthSeconds")?.as_str()?.parse().unwrap());

        let title_base = item.get("title")?;

        let mut title = title_base.get("simpleText");

        if title.is_none() {
            title = title_base.get("runs")?.as_array()?.get(0)?.get("text");
        }

        let title = title?.as_str()?.to_string();
        let thumbnail = get_thumbnail(&item, &video_id);
        let url = format!("{}{}", WATCH_URL_PREFIX, video_id);

        Some(YoutubeTrack {
            title,
            author,
            length: length.as_millis(),
            video_id,
            is_stream: false,
            url,
            thumbnail
        })
    }
}
