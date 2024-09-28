use axum::extract::{Query, State as AxumState};
use axum::Json;
use serde::Deserialize;
use crate::api::error::IntoResponseError;
use crate::api::state::State;
use crate::source::deezer::ItemType;
use crate::source::deezer::model::{DeezerAlbum, DeezerPlaylist, DeezerTrack, Either3};

#[derive(Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    id: Option<usize>,
    #[serde(default)]
    isrc: Option<String>
}

pub async fn search(
    AxumState(state): AxumState<State>,
    Query(SearchQuery { query, id, isrc }): Query<SearchQuery>
) -> Result<Json<Vec<DeezerTrack>>, IntoResponseError>
{
    match (query, id, isrc) {
        (Some(q), None, None) => state.sources.deezer.search(&q).await
            .map(Json)
            .map_err(From::from),
        (None, Some(id), None) => {
            match state.sources.deezer.get_by_id(id, ItemType::Track).await? {
                Either3::A(t) => Ok(Json(vec![t])),
                _ => unsafe { std::hint::unreachable_unchecked() }
            }
        },
        (None, None, Some(isrc)) => state.sources.deezer.get_by_isrc(isrc).await
            .map(|t| Json(vec![t]))
            .map_err(From::from),
        (None, None, None) => Err(IntoResponseError::new("None of `query`, `id` and `isrc` provided")),
        _ => Err(IntoResponseError::new("`query`, `id` and `isrc` are mutually exclusive"))
    }
}

#[derive(Deserialize)]
pub struct PlaylistQuery {
    playlist: usize,
}

pub async fn playlist(
    AxumState(state): AxumState<State>,
    Query(query): Query<PlaylistQuery>
) -> Result<Json<DeezerPlaylist>, IntoResponseError>
{
    let playlist = match state.sources.deezer.get_by_id(query.playlist, ItemType::Playlist).await? {
        Either3::B(playlist) => playlist,
        _ => unsafe { std::hint::unreachable_unchecked() }
    };

    Ok(Json(playlist))
}

#[derive(Deserialize)]
pub struct AlbumQuery {
    album: usize
}

pub async fn album(
    AxumState(state): AxumState<State>,
    Query(query): Query<AlbumQuery>
) -> Result<Json<DeezerAlbum>, IntoResponseError>
{
    let album = match state.sources.deezer.get_by_id(query.album, ItemType::Album).await? {
        Either3::C(album) => album,
        _ => unsafe { std::hint::unreachable_unchecked() }
    };

    Ok(Json(album))
}
