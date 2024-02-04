use axum::body::Body;
use axum::extract::{Query, State as AxumState};
use axum::http::StatusCode;
use axum::Json;
use axum::response::Response;
use serde::Deserialize;
use crate::api::state::State;
use crate::search::youtube::playlist::YoutubePlaylist;
use crate::search::youtube::track::YoutubeTrack;
use crate::api::APPLICATION_JSON;

#[derive(Deserialize)]
pub struct SearchQuery {
    query: String
}

pub async fn search(
    AxumState(state): AxumState<State>,
    Query(query): Query<SearchQuery>
) -> Result<Json<Vec<YoutubeTrack>>, Response> {
    match state.search.youtube.search_tracks(query.query).await {
        Ok(tracks) => Ok(Json(tracks)),
        Err(e) => Err(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(
                axum::http::header::CONTENT_TYPE,
                APPLICATION_JSON
            )
            .body(Body::from(format!(r#"{{"message": "{e}"}}"#)))
            .unwrap()
        )
    }
}

#[derive(Deserialize)]
pub struct PlaylistQuery {
    playlist_id: String
}

pub async fn playlist(
    AxumState(state): AxumState<State>,
    Query(query): Query<PlaylistQuery>
) -> Result<Json<YoutubePlaylist>, Response> {
    match state.search.youtube.get_playlist(query.playlist_id).await {
        Ok(playlist) => Ok(Json(playlist)),
        Err(e) => Err(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(
                axum::http::header::CONTENT_TYPE,
                APPLICATION_JSON
            )
            .body(Body::from(format!(r#"{{"message": "{e}"}}"#)))
            .unwrap()
        )
    }
}
