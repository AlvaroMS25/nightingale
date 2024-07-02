use std::collections::HashMap;
use std::time::{Duration, Instant};
use parking_lot::lock_api::MappedRwLockReadGuard;
use parking_lot::{RawRwLock, RwLock, RwLockReadGuard};
use rand::RngCore;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use tracing::info;
use serde_json::Value;
use songbird::input::{HttpRequest, Input};
use crate::api::error::IntoResponseError;
use crate::source::deezer::error::Error;
use crate::source::deezer::model::{DeezerAlbum, DeezerAlbumData, DeezerPlaylist, DeezerPlaylistData, DeezerTrack, Either3, GetTrackFullResponse, GetUserResponse, ItemData, JwtResponse, RequestTrackBody, Response, ResponseWithErrorAsObjects, SearchResponse, StreamRequest, StreamResponse};
use crate::source::deezer::stream::DeezerHttpStream;
use crate::source::{Playable, SourcePlayer};

pub mod model;
pub mod error;
pub mod stream;

const GET_USER_URL: &str = "https://www.deezer.com/ajax/gw-light.php?method=deezer.getUserData&input=3&api_version=1.0&api_token=";
const GET_TRACK: &str = "https://www.deezer.com/ajax/gw-light.php?method=song.getListData&input=3&api_version=1.0&api_token=";
const JWT_URL: &str = "https://auth.deezer.com/login/arl?jo=p&rto=c&i=c";
const BASE_API_URL: &str = "https://api.deezer.com/2.0/";
const SEARCH_URL: &str = "https://api.deezer.com/2.0/search?q=";
const GET_URL: &str = "https://media.deezer.com/v1/get_url";

const SECRET_KEY: &[u8; 16] = b"g4el58wc0zvf9na1";
const SECRET_IV: [u8; 8] = [0,1,2,3,4,5,6,7];

#[derive(Debug)]
pub enum ItemType {
    Track,
    Playlist,
    Album
}

impl ItemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Track => "track",
            Self::Playlist => "playlist",
            Self::Album => "album"
        }
    }
}

pub struct Initialized {
    license_token: String,
    csrf_token: String,
    media_url: String,
    cookie: String,
}

enum InitializeStatus {
    Uninit,
    Initialized(Initialized)
}

pub struct Deezer {
    http: Client,
    timeout: Duration,
    next_timeout: RwLock<Instant>,
    regexes: Box<[Regex]>,
    status: RwLock<InitializeStatus>
}

impl Deezer {
    pub fn new(client: Client) -> Self {
        let timeout = Duration::from_secs(30 * 60);
        Self {
            http: client,
            timeout,
            next_timeout: RwLock::new(Instant::now()),
            regexes: vec![
                Regex::new(r#"^https?://(?:www\.)?deezer\.com/(track|album|playlist)/(\d+)$"#).unwrap()
            ].into_boxed_slice(),
            status: RwLock::new(InitializeStatus::Uninit)
        }
    }

    pub fn can_play(&self, link: &str) -> bool {
        for regex in self.regexes.iter() {
            if regex.is_match(link) {
                return true;
            }
        }

        false
    }

    async fn maintenance(&self) -> Result<(), Error> {
        if Instant::now() > *self.next_timeout.read() {
            self.initialize().await?;
            *self.next_timeout.write() = Instant::now() + self.timeout;
        }

        Ok(())
    }

    pub async fn initialize(&self) -> Result<(), Error> {
        info!("Initializing deezer client");

        let token = {
            let mut buf = [0u8; 16];
            rand::thread_rng().fill_bytes(&mut buf);
            hex::encode(buf)
        };

        let url = format!("{GET_USER_URL}{token}");
        let res = self.http.get(url).send().await?;

        let cookie = res.headers().get_all("set-cookie")
            .iter()
            .map(|k| k.to_str().expect("Invalid header"))
            .collect::<Vec<_>>()
            .join("; ");

        let json = res.json::<GetUserResponse>().await?;

        *self.status.write() = InitializeStatus::Initialized(Initialized {
            license_token: json.results.user.options.license_token,
            csrf_token: json.results.check_form,
            media_url: json.results.url_media,
            cookie,
        });

        info!("Initialized deezer client");

        Ok(())
    }

    pub async fn search(&self, query: &str) -> Result<Vec<DeezerTrack>, Error> {
        self.maintenance().await?;

        let url = format!("{SEARCH_URL}{}", urlencoding::encode(query));
        let res = self.http.get(url).send().await?
            .json::<Response<SearchResponse>>()
            .await?
            .into_result()?;

        if res.total == 0 {
            return Ok(Vec::new());
        }

        Ok(res.data.into_iter().map(DeezerTrack::parse).collect())
    }

    pub async fn get_by_id(
        &self,
        id: usize,
        kind: ItemType
    ) -> Result<Either3<DeezerTrack, DeezerPlaylist, DeezerAlbum>, Error> {
        self.maintenance().await?;

        let res = self.http.get(format!("{BASE_API_URL}{}/{id}", kind.as_str()))
            .send()
            .await?;

        match kind {
            ItemType::Track => Ok(Either3::A(DeezerTrack::parse(res.json::<Response<ItemData>>()
                .await?
                .into_result()?
            ))),
            ItemType::Playlist => Ok(Either3::B(DeezerPlaylist::parse(res.json::<Response<DeezerPlaylistData>>()
                .await?
                .into_result()?
            ))),
            ItemType::Album => Ok(Either3::C(DeezerAlbum::parse(res.json::<Response<DeezerAlbumData>>()
                .await?
                .into_result()?)))
        }
    }

    fn url_parts(&self, url: &str) -> Option<(ItemType, usize)> {
        let capture = unsafe { self.regexes.get_unchecked(0) }.captures(url)?;

        let kind = match capture.get(1)?.as_str() {
            "track" => ItemType::Track,
            "album" => ItemType::Album,
            "playlist" => ItemType::Playlist,
            _ => return None,
        };

        let id = capture.get(2)?.as_str().parse().ok()?; // should not fail, regex is number

        Some((kind, id))
    }

    pub async fn get_link(
        &self,
        url: &str
    ) -> Result<Either3<DeezerTrack, DeezerPlaylist, DeezerAlbum>, Error>
    {
        let (kind, id) = self.url_parts(url).ok_or(Error::InvalidUrl)?;
        self.get_by_id(id, kind).await
    }

    unsafe fn get_initialized_unchecked(&self) -> MappedRwLockReadGuard<RawRwLock, Initialized> {
        RwLockReadGuard::map(self.status.read(), |status| match status {
            InitializeStatus::Initialized(s) => s,
            _ => unsafe { std::hint::unreachable_unchecked() }
        })
    }

    pub async fn create_stream(&self, track: &DeezerTrack) -> Result<DeezerHttpStream, Error> {
        self.maintenance().await?;

        let (req, license_token) = {
            // SAFETY: We are initialized for sure
            let initialized = unsafe { self.get_initialized_unchecked() };

            (
                self.http.post(format!("{GET_TRACK}{}", initialized.csrf_token))
                    .json(&RequestTrackBody {
                        sng_ids: &[track.id]
                    })
                    .header("Cookie", &initialized.cookie),
                initialized.license_token.clone()
            )
        };

        let get_track = req
            .send()
            .await?
            .json::<ResponseWithErrorAsObjects<GetTrackFullResponse>>()
            .await?
            .into_result()?;

        if get_track.results.count == 0 {
            return Err(Error::NoTrackFound(track.id));
        }


        let mut get_stream = self.http.post(GET_URL)
            .json(&StreamRequest::new(&license_token, &[&get_track.results.data[0].track_token]))
            .send()
            .await?
            .json::<StreamResponse>()
            //.json::<serde_json::Value>()
            .await?;

        if get_stream.data.is_empty()
            || get_stream.data.get(0).unwrap().media.is_empty()
            || get_stream.data.get(0).unwrap().media.get(0).unwrap().sources.is_empty()
        {
            return Err(Error::NoTrackFound(track.id));
        }

        let media_item = get_stream.data.remove(0)
            .media
            .remove(0)
            .sources
            .remove(0)
            .url;

        Ok(DeezerHttpStream {
            inner: HttpRequest::new(self.http.clone(), media_item),
            key: get_key(track.id)
        })
    }
}

fn get_key(id: usize) -> String {
    let hash_back = hex::encode(md5::compute(id.to_string()).0);
    let hash = hash_back.as_bytes();

    (0..16).fold("".to_string(), |mut acc, i| {
        acc.push((hash[i] ^ hash[i + 16] ^ SECRET_KEY[i]) as char);
        acc
    })
}

#[async_trait::async_trait]
impl SourcePlayer for Deezer {
    async fn play_url(&self, url: String) -> Result<Playable, IntoResponseError> {
        match self.url_parts(&url) {
            Some((ItemType::Track, id)) => {
                let t = match self.get_by_id(id, ItemType::Track).await? {
                    Either3::A(t) => t,
                    _ => unsafe { std::hint::unreachable_unchecked() }
                };

                let stream = self.create_stream(&t).await?;

                Ok(Playable {
                    input: Input::Lazy(Box::new(stream)),
                    meta: t.into()
                })
            },
            Some(_) => Err(IntoResponseError::new("Non-track url provided")),
            None => Err(IntoResponseError::new("Invalid URL provided"))
        }
    }
}
