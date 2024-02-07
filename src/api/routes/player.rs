use axum::response::IntoResponse;

use crate::api::extractors::player::PlayerExtractor;

pub async fn queue(PlayerExtractor(player): PlayerExtractor) -> impl IntoResponse {
    
}