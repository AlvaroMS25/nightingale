use axum::body::Body;
use axum::extract::{Query, State as AxumState};
use axum::http::StatusCode;
use axum::Json;
use axum::response::Response;
use serde::Deserialize;
use crate::api::state::State;
use crate::api::APPLICATION_JSON;
use crate::source::youtube::model::{YoutubePlaylist, YoutubeTrack};

/// Query used on [`search`] route.
#[derive(Deserialize)]
pub struct SearchQuery {
    query: String
}

/// Searches the first page of results from YouTube.
pub async fn search(
    AxumState(state): AxumState<State>,
    Query(query): Query<SearchQuery>
) -> Result<Json<Vec<YoutubeTrack>>, Response> {
    match state.sources.youtube.search_videos(query.query, 15).await {
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

/// Query used on [`playlist`] route.
#[derive(Deserialize)]
pub struct PlaylistQuery {
    playlist: String
}

/// Retrieves a playlist from the given playlist id.
pub async fn playlist(
    AxumState(state): AxumState<State>,
    Query(query): Query<PlaylistQuery>
) -> Result<Json<YoutubePlaylist>, Response> {
    match state.sources.youtube.playlist(query.playlist).await {
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
