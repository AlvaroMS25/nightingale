use axum::Router;
use axum::routing::{patch, put};
use crate::api::state::State;

mod playback;
mod gateway;

pub fn get_router() -> Router<State> {
    Router::new()
        .route("/connect", put(gateway::connect))
        .route("/disconnect", put(gateway::disconnect))
        .route("/play", put(playback::play))
}
