use serde::Serialize;
use serde_json::Value;
use crate::ext::JsonValueExt;
use super::{get_thumbnail, parse_time, WATCH_URL_PREFIX};

#[derive(Serialize)]
pub struct YoutubeTrack {
    pub title: String,
    pub author: Option<String>,
    pub length: u128,
    pub video_id: String,
    pub is_stream: bool,
    pub url: String,
    pub thumbnail: String
}

impl YoutubeTrack {
    pub fn extract_tracks(mut json: Value) -> Option<Vec<Self>> {
        let tracks = json.get_owned_object("contents")?
            .remove("sectionListRenderer")?
            .get_owned_array("contents")?
            .into_iter()
            .map(|mut item| -> Option<Vec<Value>> {
                item.get_owned_object("itemSectionRenderer")?
                    .remove("contents")?
                    .into_array()
            })
            .filter(|f| f.is_some())
            .map(|item| item.unwrap())
            .map(|values| {
                values.into_iter()
                    .map(Self::extract_one)
                    .filter(Option::is_some)
                    .map(Option::unwrap)
                    .collect::<Vec<_>>()
            })
            .fold(Vec::new(), |mut buf, tracks| {
                buf.extend(tracks);
                buf
            });

        Some(tracks)
    }

    fn extract_one(mut json: Value) -> Option<Self> {
        let json = json.get_mut("compactVideoRenderer")?;
        let data = json.as_object_mut()?;

        if data.get("lengthText").is_none() {
            return None;
        }

        let title = data.remove("title")?
            .get_owned_array("runs")?
            .remove(0)
            .get_owned_string("text")?;

        let author = data.remove("longBylineText")?
            .get_owned_array("runs")?
            .remove(0)
            .get_owned_string("text");

        let seconds = data.remove("lengthText")?
            .get_owned_array("runs")?
            .remove(0)
            .get_owned_string("text")?;

        let seconds = parse_time(&seconds).unwrap();

        let video_id = data.remove("videoId")?.into_string()?;

        let thumbnail = get_thumbnail(&json, &video_id);
        let url = format!("{}{}", WATCH_URL_PREFIX, video_id);

        Some(Self {
            title,
            author,
            length: seconds.as_millis(),
            video_id,
            is_stream: false,
            url,
            thumbnail
        })
    }
}
