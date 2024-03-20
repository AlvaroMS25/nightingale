pub mod track;
pub mod playlist;

use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;
use const_format::concatcp;
use reqwest::Client;
use serde_json::{json, Value};
use crate::source::yt::playlist::YoutubePlaylist;
use crate::source::yt::track::YoutubeTrack;

const CLIENT_ANDROID_NAME: &str = "ANDROID";
const CLIENT_ANDROID_VERSION: &str = "18.06.35";
const ANDROID_SDK_VERSION: u8 = 30;
const DEFAULT_ANDROID_VERSION: &str = "11";
const SEARCH_PARAMS: &str = "EgIQAUICCAE=";
const INNERTUBE_ANDROID_API_KEY: &str = "AIzaSyA8eiZmM1FaDVjRy-df2KTyQ_vz_yYM39w";
const YOUTUBE_API_ORIGIN: &str = "https://youtubei.googleapis.com";
const YOUTUBE_BASE_URL: &str = concatcp!(YOUTUBE_API_ORIGIN, "/youtubei/v1");
const SEARCH_URL: &str = concatcp!(YOUTUBE_BASE_URL, "/search?key=", INNERTUBE_ANDROID_API_KEY);
const WATCH_URL_PREFIX: &str = "https://www.youtube.com/watch?v=";
const YOUTUBE_BROWSE_URL: &str = concatcp!(YOUTUBE_BASE_URL, "/browse");


/// Search client for youtube.
pub struct YoutubeSearch {
    http: Client,
    default_body: Value
}

impl YoutubeSearch {
    pub fn new() -> Self {
        let http = Client::builder()
            .user_agent(format!(
                "com.google.android.youtube/{} (Linux; U; Android {}) gzip",
                CLIENT_ANDROID_VERSION,
                DEFAULT_ANDROID_VERSION
            ))
            .build()
            .expect("Failed to build youtube search HTTP client");

        Self {
            http,
            default_body: Self::get_default_body()
        }
    }

    fn get_default_body() -> Value {
        json!({
            "context": {
                "client": {
                    "clientName": CLIENT_ANDROID_NAME,
                    "clientVersion": CLIENT_ANDROID_VERSION,
                    "androidSdkVersion": ANDROID_SDK_VERSION,
                    "screenDensityFloat": 1,
                    "screenHeightPoints": 1080,
                    "screenPixelDensity": 1,
                    "screenWidthPoints": 1920
                }
            }
        })
    }

    pub async fn search_tracks(&self, query: impl ToString) -> Result<Vec<YoutubeTrack>, reqwest::Error> {
        let mut body = self.default_body.clone();
        body["query"] = json!(query.to_string());
        body["params"] = json!(SEARCH_PARAMS);

        let output = self.http.post(SEARCH_URL)
            .json(&body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(YoutubeTrack::extract_tracks(output).unwrap_or_default())
    }

    pub async fn get_playlist(&self, playlist_id: impl Display) -> Result<YoutubePlaylist, reqwest::Error>{
        let mut body = self.default_body.clone();

        {
            let object = body.as_object_mut().unwrap();

            object.insert(String::from("browseId"), json!(format!("VL{playlist_id}")));
        }

        let output = self.http.post(YOUTUBE_BROWSE_URL)
            .json(&body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        let name = output["header"]["playlistHeaderRenderer"]["title"]["runs"][0]["text"].to_string();
        let items = output["contents"]["singleColumnBrowseResultsRenderer"]["tabs"][0]["tabRenderer"]["content"]["sectionListRenderer"]["contents"][0]["playlistVideoListRenderer"].clone();

        Ok(YoutubePlaylist {
            name,
            tracks: self.get_playlist_inner(items).await?
        })
    }

    async fn get_playlist_inner(&self, mut object: Value) -> Result<Vec<YoutubeTrack>, reqwest::Error> {
        let mut tracks = Vec::new();

        while let Some(token) = YoutubePlaylist::parse(object, &mut tracks) {
            let mut body = self.default_body.clone();
            body["continuation"] = json!(token);

            let mut output = self.http.post(YOUTUBE_BROWSE_URL)
                .json(&body)
                .send()
                .await?
                .json::<Value>()
                .await?;

            let Some(Some(map)) = output.get_mut("continuationContents").map(|item| item.as_object_mut()) else { break; };
            let Some(continuation) = map.remove("playlistVideoListContinuation") else { break; };

            object = continuation;
        }

        Ok(tracks)
    }
}

pub fn get_thumbnail(item: &Value, video_id: &String) -> String {
    let thumbnails_array = item.get("thumbnail")
        .map(|f| f.get("thumbnails").map(|t| t.as_array()))
        .flatten()
        .flatten();

    let last = thumbnails_array.map(|values| values.iter().last().map(|last| last.as_str()))
        .flatten()
        .flatten();

    match last {
        Some(item) if item.contains("maxresdefault") => item.to_string(),
        _ => format!("https://i.ytimg.com/vi/{video_id}/maxresdefault.jpg")
    }
}

pub fn parse_time(time: &str) -> Result<Duration, <u64 as FromStr>::Err> {
    let mut split = time.split(":");
    let minutes = split.next().unwrap().parse::<u64>()?;
    let seconds = split.next().unwrap().parse::<u64>()?;

    Ok(Duration::from_secs((minutes * 60) + seconds))
}
