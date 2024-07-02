use axum::Router;
use axum::routing::get;
use crate::api::state::State;

mod youtube;
mod deezer;

/// Search related routes.
pub fn get_router() -> Router<State> {
    Router::new()
        .nest("/youtube", Router::new()
            .route("/search", get(youtube::search))
            .route("/playlist", get(youtube::playlist))
        )
        .nest("/deezer", Router::new()
            .route("/search", get(deezer::search))
            .route("/playlist", get(deezer::playlist))
            .route("/album", get(deezer::album))
        )
}
