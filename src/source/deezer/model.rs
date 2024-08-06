use std::collections::HashMap;
use std::time::Duration;
use phf::phf_map;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use songbird::input::AuxMetadata;
use crate::source::deezer::error::Error;

#[derive(Deserialize, Debug)]
pub struct ResponseError {
    #[serde(rename = "type")]
    pub kind: String,
    pub code: usize,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct GetUserResponse {
    pub error: Vec<ResponseError>,
    //pub error: Option<ResponseError>,
    pub results: Results
}

#[derive(Deserialize, Debug)]
pub struct Results {
    #[serde(rename = "USER")]
    pub user: User,
    #[serde(rename = "checkForm")]
    pub check_form: String,
    #[serde(rename = "URL_MEDIA")]
    pub url_media: String
}

#[derive(Deserialize, Debug)]
pub struct User {
    #[serde(rename = "OPTIONS")]
    pub options: UserOptions
}

#[derive(Deserialize, Debug)]
pub struct UserOptions {
    pub license_token: String
}

#[derive(Deserialize, Debug)]
pub struct JwtResponse {
    pub jwt: String
}

#[derive(Deserialize, Debug)]
pub struct SearchResponse {
    pub data: Vec<ItemData>,
    pub total: usize,
    #[serde(default)]
    pub next: Option<String>
}

#[derive(Deserialize, Debug)]
pub enum Response<T> {
    #[serde(rename = "error")]
    Error(ResponseError),
    #[serde(untagged)]
    Ok(T),
}

impl<T> Response<T> {
    pub fn into_result(self) -> Result<T, ResponseError> {
        match self {
            Self::Error(e) => Err(e),
            Self::Ok(o) => Ok(o)
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum ResponseWithErrorAsObjects<T> {
    #[serde(untagged)]
    Error {
        error: HashMap<String, Value>
    },
    #[serde(untagged)]
    Ok(T)
}

impl<T> ResponseWithErrorAsObjects<T> {
    pub fn into_result(self) -> Result<T, Error> {
        match self {
            Self::Error {error} => Err(Error::Dynamic(error)),
            Self::Ok(v) => Ok(v)
        }
    }
}

pub struct ResponseWithErrors<T> {
    pub errors: Vec<ResponseError>,
    pub data: T,
}

#[derive(Deserialize, Debug)]
pub struct ItemData {
    pub id: usize,
    pub artist: Artist,
    pub duration: u64,
    pub title: String,
    pub link: String,
    #[serde(default)]
    pub album: Option<Album>,
    #[serde(default)]
    pub isrc: Option<String>
}

#[derive(Deserialize, Debug, Default)]
pub struct Artist {
    pub name: String
}

#[derive(Deserialize, Debug)]
pub struct Album {
    pub cover: String,
}

#[derive(Deserialize, Debug)]
pub struct Data<T> {
    pub data: T
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeezerTrack {
    pub id: usize,
    pub author: String,
    pub length: u64,
    pub title: String,
    pub uri: String,
    pub artwork_url: Option<String>,
    pub isrc: Option<String>
}

impl DeezerTrack {
    pub fn parse(item: ItemData) -> Self {
        DeezerTrack {
            id: item.id,
            author: item.artist.name,
            length: item.duration * 1000,
            title: item.title,
            uri: item.link,
            artwork_url: item.album.map(|a| a.cover),
            isrc: item.isrc
        }
    }
}

impl From<DeezerTrack> for AuxMetadata {
    fn from(value: DeezerTrack) -> Self {
        AuxMetadata {
            track: None,
            artist: Some(value.author),
            duration: Some(Duration::from_millis(value.length)),
            title: Some(value.title),
            source_url: Some(value.uri),
            thumbnail: value.artwork_url,
            ..Default::default()
        }
    }
}

pub struct Genres {
    data: Vec<GenreInner>
}

#[derive(Deserialize, Debug)]
pub struct GenreInner {
    pub id: usize,
    pub name: String,
    pub picture: String,
    #[serde(rename = "type")]
    pub kind: String
}

#[derive(Deserialize, Debug)]
pub struct DeezerAlbumData {
    pub id: usize,
    pub title: String,
    pub link: String,
    pub cover_xl: String,
    pub nb_tracks: usize,
    pub duration: usize,
    pub artist: Artist,
    pub tracks: Data<Vec<ItemData>>
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeezerAlbum {
    pub id: usize,
    pub title: String,
    pub link: String,
    pub cover_xl: String,
    pub track_number: usize,
    pub duration: usize,
    pub author: String,
    pub tracks: Vec<DeezerTrack>
}

impl DeezerAlbum {
    pub fn parse(item: DeezerAlbumData) -> Self {
        Self {
            id: item.id,
            title: item.title,
            link: item.link,
            cover_xl: item.cover_xl,
            track_number: item.nb_tracks,
            duration: item.duration * 1000,
            author: item.artist.name,
            tracks: item.tracks.data.into_iter().map(DeezerTrack::parse).collect()
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Creator {
    name: String
}

#[derive(Deserialize, Debug)]
pub struct DeezerPlaylistData {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub public: bool,
    pub link: String,
    pub picture_xl: String,
    pub nb_tracks: usize,
    pub duration: usize,
    pub creator: Creator,
    pub tracks: Data<Vec<ItemData>>
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeezerPlaylist {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub public: bool,
    pub link: String,
    pub picture_xl: String,
    pub track_number: usize,
    pub duration: usize,
    pub creator: String,
    pub tracks: Vec<DeezerTrack>
}

impl DeezerPlaylist {
    pub fn parse(item: DeezerPlaylistData) -> Self {
        Self {
            id: item.id,
            title: item.title,
            description: item.description,
            public: item.public,
            link: item.link,
            picture_xl: item.picture_xl,
            track_number: item.nb_tracks,
            duration: item.duration * 1000,
            creator: item.creator.name,
            tracks: item.tracks.data.into_iter().map(DeezerTrack::parse).collect()
        }
    }
}

#[derive(Serialize, Debug)]
pub enum Either3<A, B, C> {
    A(A),
    B(B),
    C(C)
}

#[derive(Serialize, Debug)]
pub struct RequestTrackBody<'a> {
    pub sng_ids: &'a [usize]
}

#[derive(Deserialize, Debug)]
pub struct GetTrackFullResponse {
    pub error: Vec<ResponseError>,
    pub results: GetTrackResponse
}

#[derive(Deserialize, Debug)]
pub struct GetTrackResponse {
    pub count: usize,
    pub data: Vec<GetTrackResponseInner>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct GetTrackResponseInner {
    pub track_token: String
}

#[derive(Serialize, Debug)]
pub struct StreamRequest<'a> {
    license_token: &'a str,
    media: &'static [Media],
    track_tokens: &'a [&'a str]
}

const DEFAULT_MEDIA: Media = Media {
    kind: "FULL",
    formats: &[
        phf_map! {
            "cipher" => "BF_CBC_STRIPE",
            "format" => "FLAC"
        },
        phf_map! {
            "cipher" => "BF_CBC_STRIPE",
            "format" => "MP3_256"
        },
        phf_map! {
            "cipher" => "BF_CBC_STRIPE",
            "format" => "MP3_128"
        },
        phf_map! {
            "cipher" => "BF_CBC_STRIPE",
            "format" => "MP3_MISC"
        }
    ]
};

#[derive(Serialize, Debug)]
pub struct Media {
    #[serde(rename = "type")]
    kind: &'static str,
    formats: &'static [phf::Map<&'static str, &'static str>]
}

impl<'a> StreamRequest<'a> {
    pub fn new(token: &'a str, track_tokens: &'a [&'a str]) -> Self {
        Self {
            license_token: token,
            media: &[DEFAULT_MEDIA],
            track_tokens
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct StreamResponse {
    pub data: Vec<StreamResponseData>
}

#[derive(Deserialize, Debug)]
pub struct StreamResponseData {
    pub media: Vec<StreamResponseMedia>
}

#[derive(Deserialize, Debug)]
pub struct StreamResponseMedia {
    pub sources: Vec<StreamResponseSources>
}

#[derive(Deserialize, Debug)]
pub struct StreamResponseSources {
    pub provider: String,
    pub url: String,
}
