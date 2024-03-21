use std::num::NonZeroU64;
use std::sync::Arc;
use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use axum::http::StatusCode;
use futures_util::TryFutureExt;
use uuid::Uuid;
use crate::api::error::IntoResponseError;
use crate::api::session::Session;
use crate::api::state::State;

const MISSING_SESSION_ID: &str = "Missing session ID";
const SESSION_NOT_PRESENT: &str = "Session not present";

/// Extractor that takes a session from the url parameters and resolves to the corresponding session,
/// if the session is not provided or invalid, it returns a 400 Bad request with the corresponding
/// error message.
pub struct SessionExtractor(pub Arc<Session>);

#[async_trait::async_trait]
impl FromRequestParts<State> for SessionExtractor {
    type Rejection = IntoResponseError;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let Path(id) = <Path<Uuid> as FromRequestParts<State>>::from_request_parts(parts, state)
            .map_err(|_| IntoResponseError::new(MISSING_SESSION_ID)
                .with_status(StatusCode::BAD_REQUEST))
            .await?;

        Self::from_id(id, state)
    }
}

impl SessionExtractor {
    pub fn from_id(id: Uuid, state: &State) -> Result<Self, IntoResponseError> {
        let Some(session) = state.instances.get(&id) else {
            return Err(IntoResponseError::new(SESSION_NOT_PRESENT)
                .with_status(StatusCode::BAD_REQUEST)
            )
        };

        Ok(SessionExtractor(Arc::clone(session.value())))
    }
}

/// Just like [`SessionExtractor`] but this extractor takes the session and guild
/// parameters from the url instead of only taking the session id.
pub struct SessionWithGuildExtractor {
    pub session: Arc<Session>,
    pub guild: NonZeroU64
}

#[async_trait::async_trait]
impl FromRequestParts<State> for SessionWithGuildExtractor {
    type Rejection = IntoResponseError;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let Path((id, guild)) =
            <Path<(Uuid, NonZeroU64)> as FromRequestParts<State>>::from_request_parts(parts, state)
            .map_err(|_| IntoResponseError::new(super::player::MISSING_ID)
                .with_status(StatusCode::BAD_REQUEST))
            .await?;

        let SessionExtractor(session) = SessionExtractor::from_id(id, state)?;

        Ok(Self {
            session,
            guild
        })
    }
}
