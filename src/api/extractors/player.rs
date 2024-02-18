use std::num::NonZeroU64;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::{FromRequestParts, Path, Query};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Response;
use futures_util::TryFutureExt;
use serde::Deserialize;
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::api::extractors::session::SessionExtractor;
use crate::api::state::State;
use crate::playback::player::Player;

const PLAYER_NON_EXISTENT: &str = r#"{"message": "The player does not exist"}"#;
pub(super) const MISSING_ID: &str = r#"{"message": "Missing guild or session ID"}"#;

/// Extractor that takes a guild id from the url parameters and resolves to the corresponding player,
/// if the guild is not provided or the player is not available, returns a 400 Bad request
/// response with the corresponding error message.
///
/// This extractor uses the [`SessionExtractor`] under the hood, and needs it to resolve first.
pub struct PlayerExtractor {
    pub player: Arc<Mutex<Player>>,
    pub guild: NonZeroU64
}

#[async_trait::async_trait]
impl FromRequestParts<State> for PlayerExtractor {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let Path((session, guild)) =
            <Path<(Uuid, NonZeroU64)> as FromRequestParts<State>>::from_request_parts(parts, state)
                .map_err(|_| Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(
                        axum::http::header::CONTENT_TYPE,
                        super::super::APPLICATION_JSON
                    )
                    .body(Body::from(MISSING_ID))
                    .unwrap()
                )
                .await?;

        let SessionExtractor(session) = SessionExtractor::from_id(session, state)?;

        let Some(player) = session.playback.get_player(guild) else {
            return Err(
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(
                        axum::http::header::CONTENT_TYPE,
                        super::super::APPLICATION_JSON
                    )
                    .body(Body::from(PLAYER_NON_EXISTENT))
                    .unwrap()
            );
        };

        Ok(PlayerExtractor {
            player,
            guild
        })
    }
}
