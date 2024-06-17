use std::net::SocketAddr;

use axum::Router;
use axum::routing::get;
use axum_server::tls_rustls::RustlsConfig;
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
pub mod error;
mod serde;

const APPLICATION_JSON: &str = "application/json";

/// Starts the Axum Http server.
pub async fn start_http(config: Config) -> Result<(), std::io::Error> {
    info!("Creating HTTP server");

    let state = State::new(&config);

    let mut router = Router::new()
        .route("/ws", get(websocket::connect))
        .route("/ws/resume/:session", get(websocket::resume))
        .nest("/api/v1", routes::get_router())
        .layer(RequireAuth(config.server.password.clone()))
        .with_state(state);

    if let Some(filter) = config.server.filter_ips {
        router = router.layer(IpFilter(filter));
    }

    info!(
        "Starting HTTP{} server on {}:{}",
        if config.server.ssl.is_some() { "S" } else { "" },
        config.server.address,
        config.server.port
    );

    let addr = format!("{}:{}", config.server.address, config.server.port).parse().unwrap();
    if let Some(ssl_config) = config.server.ssl {
        axum_server::bind_rustls(
            addr,
            RustlsConfig::from_pem_file(ssl_config.cert_path, ssl_config.key_path).await?
        ).serve(router.into_make_service_with_connect_info::<SocketAddr>()).await
    } else {
        axum_server::Server::bind(addr)
            .serve(router.into_make_service_with_connect_info::<SocketAddr>())
            .await
    }

}
