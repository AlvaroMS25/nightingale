use axum::extract::State as AxumState;
use axum::response::IntoResponse;
use crate::api::state::State;

/// TODO: implement route.
pub async fn prometheus_metrics(AxumState(_state): AxumState<State>) -> impl IntoResponse {
    todo!()
}