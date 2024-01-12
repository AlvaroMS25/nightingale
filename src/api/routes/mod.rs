use axum::Router;
use axum::routing::{patch, put};
use crate::api::state::State;

mod connect;
mod disconnect;
mod play;

pub fn get_router() -> Router<State> {
    Router::new()
        .route("/connect", put(connect::connect_voice))
        .route("/disconnect", put(disconnect::disconnect_voice))
}
