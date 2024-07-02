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
    id: Option<usize>
}

pub async fn search(
    AxumState(state): AxumState<State>,
    Query(SearchQuery { query, id }): Query<SearchQuery>
) -> Result<Json<Vec<DeezerTrack>>, IntoResponseError>
{
    match (query, id) {
        (Some(_), Some(_)) => Err(IntoResponseError::new("Both `query` and `id` provided")),
        (None, None) => Err(IntoResponseError::new("Neither `query` nor `id` provided")),
        (Some(q), None) => state.sources.deezer.search(&q).await
            .map(Json)
            .map_err(From::from),
        (None, Some(id)) => {
            match state.sources.deezer.get_by_id(id, ItemType::Track).await? {
                Either3::A(t) => Ok(Json(vec![t])),
                _ => unsafe { std::hint::unreachable_unchecked() }
            }
        }
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
