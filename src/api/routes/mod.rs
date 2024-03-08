use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, get, patch, post, put};
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
                .route("/connect", put(player::connect))
                .route("/disconnect", delete(player::disconnect))
                .route("/info", get(player::info))
                .route("/play", post(player::play))
                .route("/pause", patch(player::pause))
                .route("/resume", patch(player::resume))
                .route("/set_volume/:volume", patch(player::volume))
                .nest("/queue", Router::new()

                )
            )
        )
}

/*
TODO: reorganize routes

    /ws:
        - / -> connect to websocket
        - /resume -> resume a previous session
    /api/v1:
        - /search/... -> search on sources
        - /info(?session) -> system information (about session or all of them)
        - /prometheus -> prometheus metrics

        - /{session}:
            - /connect -> connect to voice
            - /players/{guild}
                - /info (get)
                - /play (post)
                - /pause (patch)
                - /resume (patch)
                - /set_volume/<vol> (patch)
                - /queue:
                    - / (patch)
                    - /clear (put)
                - /disconnect (delete)




 */