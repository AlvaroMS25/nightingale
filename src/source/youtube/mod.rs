pub mod model;

use reqwest::Client;
use rusty_ytdl as ytdl;
use rusty_ytdl::{RequestOptions, VideoOptions, VideoQuality, VideoSearchOptions};
use rusty_ytdl::search::{SearchResult, SearchType};
use serde::Serialize;
use crate::source::StringError;
use ytdl::search::YouTube as RustyYoutube;
use model::*;

pub struct Youtube {
    search: RustyYoutube,
    video_options: VideoOptions,
    request_options: RequestOptions,
}

impl Youtube {
    pub fn new(http: Client) -> Result<Self, StringError> {
        let request_options = RequestOptions {
            client: Some(http),
            ..Default::default()
        };

        Ok(Self {
            search: RustyYoutube::new_with_options(&request_options)?,
            video_options: VideoOptions {
                quality: VideoQuality::HighestAudio,
                filter: VideoSearchOptions::Audio,
                ..Default::default()
            },
            request_options
        })
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
        Ok(self.search.search_one(query, Some(&ytdl::search::SearchOptions {
            search_type: SearchType::Video,
            safe_search: false,
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
}
