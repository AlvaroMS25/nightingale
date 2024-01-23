use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{patch, put};
use crate::api::state::State;

mod playback;
mod gateway;

pub fn get_router() -> Router<State> {
    Router::new()
        .route("/connect", put(gateway::connect))
        .route("/disconnect", put(gateway::disconnect))
        .nest("/playback", Router::new()
            .route("/play", put(playback::play).layer(DefaultBodyLimit::disable()))
            .route("/pause", put(playback::pause))
            .route("/resume", put(playback::resume))
            .route("/volume/:vol", patch(playback::volume))
        )
}
