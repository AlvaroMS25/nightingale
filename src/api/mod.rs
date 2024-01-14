use axum::{Router, ServiceExt};
use axum::routing::get;
use tracing::info;
use crate::api::state::State;
use crate::config::Config;

mod state;
pub mod model;
mod tri;
mod websocket;
mod session;
mod routes;
mod extractors;
mod metrics;

const APPLICATION_JSON: &str = "application/json";

pub async fn start_http(config: Config) -> Result<(), std::io::Error> {
    let router = Router::new()
        .route("/ws", get(websocket::connect))
        .route("/ws/resume", get(websocket::resume))
        .nest("/api/v1", routes::get_router())
        .with_state(State::new());

    info!("Starting server on {}:{}", config.server.address, config.server.port);

    axum_server::Server::bind(format!("{}:{}", config.server.address, config.server.port).parse().unwrap())
        .serve(router.into_make_service())
        .await
}
