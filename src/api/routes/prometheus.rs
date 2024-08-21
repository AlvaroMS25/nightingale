use axum::extract::State as AxumState;
use axum::response::IntoResponse;
use crate::api::state::State;
use crate::metrics::metrics;

pub async fn prometheus_metrics(AxumState(state): AxumState<State>) -> impl IntoResponse {
    metrics().build_response()
}