use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, get, patch, post, put};
use crate::api::state::State;

mod playback;
mod gateway;
mod prometheus;
mod info;
mod search;
mod player;

/// API routes.
pub fn get_router() -> Router<State> {
    Router::new()
        .route("/connect", put(gateway::connect))
        .route("/disconnect", delete(gateway::disconnect))
        .nest("/playback", Router::new()
            .route("/play", post(playback::play).layer(DefaultBodyLimit::disable()))
            .route("/pause", patch(playback::pause))
            .route("/resume", patch(playback::resume))
            .route("/volume/:vol", patch(playback::volume))
        )
        .nest("/search", search::get_router())
        .route("/info", get(info::info))
        .nest("/player", Router::new()
            .route("/", get(player::player))
        )
        .route("/prometheus", get(prometheus::prometheus_metrics))
}
