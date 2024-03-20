pub mod model;

use reqwest::Client;
use rusty_ytdl as ytdl;
use rusty_ytdl::{RequestOptions, VideoOptions, VideoQuality, VideoSearchOptions};
use rusty_ytdl::search::{SearchOptions, SearchResult, SearchType};
use serde::Serialize;
use songbird::input::{AuxMetadata, HttpRequest};
use crate::source::{Playable, SourcePlayer, StringError};
use ytdl::search::YouTube as RustyYoutube;
use model::*;

pub struct Youtube {
    search: RustyYoutube,
    video_options: VideoOptions,
    request_options: RequestOptions,
    http: Client
}

impl Youtube {
    pub fn new(http: Client) -> Self {
        let request_options = RequestOptions {
            client: Some(http.clone()),
            ..Default::default()
        };

        Self {
            search: RustyYoutube::new_with_options(&request_options).unwrap(), // can't fail
            video_options: VideoOptions {
                quality: VideoQuality::HighestAudio,
                filter: VideoSearchOptions::Audio,
                ..Default::default()
            },
            request_options,
            http
        }
    }

    pub async fn search_videos(
        &self,
        query: String,
        limit: u64
    ) -> Result<Vec<YoutubeTrack>, StringError> {
        Ok(self.search.search(query, Some(&ytdl::search::SearchOptions {
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

    pub async fn search_video(&self, query: String) -> Result<Option<YoutubeTrack>, StringError> {
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

    pub async fn playlist(&self, playlist: String) -> Result<Option<YoutubePlaylist>, StringError> {
        Ok(self.search.search_one(playlist, Some(&SearchOptions {
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
            }))
    }
}

#[async_trait::async_trait]
impl SourcePlayer for Youtube {
    async fn play_url(&self, url: String) -> Result<Playable, StringError> {
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
