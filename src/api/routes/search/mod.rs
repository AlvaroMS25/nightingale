use axum::Router;
use axum::routing::get;
use crate::api::state::State;

mod youtube;

pub fn get_router() -> Router<State> {
    Router::new()
        .nest("/youtube", Router::new()
            .route("/search", get(youtube::search))
            .route("/playlist", get(youtube::playlist))
        )
}
