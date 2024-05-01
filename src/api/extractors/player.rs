use std::num::NonZeroU64;
use std::sync::Arc;
use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use axum::http::StatusCode;
use futures_util::TryFutureExt;
use uuid::Uuid;
use crate::api::error::IntoResponseError;
use crate::api::extractors::session::SessionExtractor;
use crate::api::state::State;
use crate::mutex::TicketedMutex;
use crate::playback::player::Player;

const PLAYER_NON_EXISTENT: &str = "The player does not exist";
pub(super) const MISSING_ID: &str = "Missing guild or session ID";

/// Extractor that takes a guild id from the url parameters and resolves to the corresponding player,
/// if the guild is not provided or the player is not available, returns a 400 Bad request
/// response with the corresponding error message.
///
/// This extractor uses the [`SessionExtractor`] under the hood, and needs it to resolve first.
pub struct PlayerExtractor {
    pub player: Arc<TicketedMutex<Player>>,
    pub guild: NonZeroU64
}

#[async_trait::async_trait]
impl FromRequestParts<State> for PlayerExtractor {
    type Rejection = IntoResponseError;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let Path((session, guild)) =
            <Path<(Uuid, NonZeroU64)> as FromRequestParts<State>>::from_request_parts(parts, state)
                .map_err(|_| IntoResponseError::new(MISSING_ID).with_status(StatusCode::BAD_REQUEST))
                .await?;

        Self::from_id(session, state, guild)
    }
}

impl PlayerExtractor {
    pub fn from_id(session: Uuid, state: &State, guild: NonZeroU64) -> Result<Self, IntoResponseError> {
        let SessionExtractor(session) = SessionExtractor::from_id(session, state)?;

        let Some(player) = session.playback.get_player(guild) else {
            return Err(IntoResponseError::new(PLAYER_NON_EXISTENT)
                .with_status(StatusCode::BAD_REQUEST)
            )
        };

        Ok(PlayerExtractor {
            player,
            guild
        })
    }
}
