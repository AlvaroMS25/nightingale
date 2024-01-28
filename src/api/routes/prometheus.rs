use axum::extract::State as AxumState;
use axum::response::IntoResponse;
use crate::api::state::State;

pub async fn prometheus_metrics(AxumState(state): AxumState<State>) -> impl IntoResponse {
    todo!()
}