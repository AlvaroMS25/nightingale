pub mod model;

use regex::Regex;
use reqwest::Client;
use rusty_ytdl as ytdl;
use rusty_ytdl::{RequestOptions, VideoOptions, VideoQuality, VideoSearchOptions};
use rusty_ytdl::search::{SearchOptions, SearchResult, SearchType};
use songbird::input::{AuxMetadata, HttpRequest};
use crate::source::{IntoResponseError, Playable, SourcePlayer};
use ytdl::search::YouTube as RustyYoutube;
use model::*;

pub struct Youtube {
    search: RustyYoutube,
    video_options: VideoOptions,
    http: Client,
    regexes: Box<[Regex]> // We use a boxed slice to add more regexes if needed later
}

impl Youtube {
    pub fn new(http: Client) -> Self {
        Self {
            search: RustyYoutube::new_with_options(&RequestOptions {
                client: Some(http.clone()),
                ..Default::default()
            }).unwrap(), // can't fail
            video_options: VideoOptions {
                quality: VideoQuality::HighestAudio,
                filter: VideoSearchOptions::Audio,
                ..Default::default()
            },
            http,
            regexes: vec![
                Regex::new(r#"^((?:https?:)?\/\/)?((?:www|m|music)\.)?((?:youtube\.com|youtu.be))(\/(?:[\w\-]+\?v=|embed\/|v\/)?)([\w\-]+)(\S+)?$"#).unwrap(),
            ].into_boxed_slice()
        }
    }

    pub fn can_play(&self, url: &str) -> bool {
        for regex in self.regexes.iter() {
            if regex.is_match(url) {
                return true;
            }
        }

        false
    }

    pub async fn search_videos(
        &self,
        query: String,
        limit: u64
    ) -> Result<Vec<YoutubeTrack>, IntoResponseError> {
        Ok(self.search.search(query, Some(&SearchOptions {
            limit,
            search_type: SearchType::Video,
            safe_search: false
        }))
            .await?
            .into_iter()
            .filter_map(|res| {
                if let SearchResult::Video(v) = res {
                    Some(v)
                } else {
                    None
                }
            })
            .map(Into::into)
            .collect::<Vec<_>>())
    }

    #[allow(unused)]
    pub async fn search_video(&self, query: String) -> Result<Option<YoutubeTrack>, IntoResponseError> {
        Ok(self.search.search_one(query, Some(&SearchOptions {
            search_type: SearchType::Video,
            ..Default::default()
        }))
            .await?
            .and_then(|res| {
                if let SearchResult::Video(v) = res {
                    Some(v.into())
                } else {
                    None
                }
            }))
    }

    pub async fn playlist(&self, playlist: String) -> Result<YoutubePlaylist, IntoResponseError> {
        self.search.search_one(playlist, Some(&SearchOptions {
            search_type: SearchType::Playlist,
            ..Default::default()
        }))
            .await?
            .and_then(|res| {
                if let SearchResult::Playlist(p) = res {
                    Some(p.into())
                } else {
                    None
                }
            })
            .ok_or_else(|| IntoResponseError::new("No playlist found"))
    }
}

#[async_trait::async_trait]
impl SourcePlayer for Youtube {
    async fn play_url(&self, url: String) -> Result<Playable, IntoResponseError> {
        let video = ytdl::Video::new_with_options(url, self.video_options.clone())?;
        let info = video.get_info().await?;

        let mut format = ytdl::choose_format(info.formats.as_slice(), &self.video_options)?;
        let url = std::mem::take(&mut format.url); // only used to create input

        let meta = AuxMetadata::try_from(WrapInfo(info.video_details, format))?;

        let req = HttpRequest::new(self.http.clone(), url);

        Ok(Playable {
            input: req.into(),
            meta
        })
    }
}
