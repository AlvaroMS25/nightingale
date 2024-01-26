use std::net::SocketAddr;

use axum::Router;
use axum::routing::get;
use tracing::info;
use layers::auth::RequireAuth;
use crate::api::layers::ip::IpFilter;
use crate::api::state::State;
use crate::config::Config;

mod state;
pub mod model;
mod tri;
mod websocket;
pub mod session;
mod routes;
mod extractors;
mod layers;

const APPLICATION_JSON: &str = "application/json";

pub async fn start_http(config: Config) -> Result<(), std::io::Error> {
    info!("Creating HTTP server");

    let state = State::new();

    let mut router = Router::new()
        .route("/ws", get(websocket::connect))
        .route("/ws/resume", get(websocket::resume))
        .nest("/api/v1", routes::get_router())
        .layer(RequireAuth(config.server.password.clone()))
        .with_state(state);

    if let Some(filter) = config.server.filter_ips {
        router = router.layer(IpFilter(filter));
    }

    info!(
        "Starting HTTP{} server on {}:{}",
        if config.server.ssl.map(|s| s.enable).unwrap_or(false) { "S" } else { "" },
        config.server.address,
        config.server.port
    );

    axum_server::Server::bind(format!("{}:{}", config.server.address, config.server.port).parse().unwrap())
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await
}
