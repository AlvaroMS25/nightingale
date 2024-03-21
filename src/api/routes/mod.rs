use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, patch, post};
use crate::api::state::State;

mod prometheus;
mod info;
mod search;
mod player;

/// API routes.
pub fn get_router() -> Router<State> {
    Router::new()
        .nest("/info", Router::new()
            .route("/", get(info::info))
            .route("/:session", get(info::info))
        )
        .route("/prometheus", get(prometheus::prometheus_metrics))
        .nest("/search", search::get_router())
        .nest("/:session", Router::new()
            .nest("/players/:guild", Router::new()
                .route("/update", patch(player::update))
                .route("/info", get(player::info))
                .route("/play", post(player::play).layer(DefaultBodyLimit::disable()))
                .route("/pause", patch(player::pause))
                .route("/resume", patch(player::resume))
                .route("/set_volume/:volume", patch(player::volume))
                .nest("/queue", Router::new()

                )
            )
        )
}
