use std::num::NonZeroU64;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::{FromRequestParts, Path, Query};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Response;
use futures_util::TryFutureExt;
use serde::Deserialize;
use uuid::Uuid;
use crate::api::session::Session;
use crate::api::state::State;

const MISSING_SESSION_ID: &str = r#"{"message": "Missing session ID"}"#;
const SESSION_NOT_PRESENT: &str = r#"{"message": "Session not present"}"#;

/// Extractor that takes a session from the url parameters and resolves to the corresponding session,
/// if the session is not provided or invalid, it returns a 400 Bad request with the corresponding
/// error message.
pub struct SessionExtractor(pub Arc<Session>);

#[async_trait::async_trait]
impl FromRequestParts<State> for SessionExtractor {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let Path(id) = <Path<Uuid> as FromRequestParts<State>>::from_request_parts(parts, state)
            .map_err(|e| Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(
                        axum::http::header::CONTENT_TYPE,
                        super::super::APPLICATION_JSON
                    )
                    .body(Body::from(MISSING_SESSION_ID))
                    .unwrap()
            )
            .await?;

        Self::from_id(id, state)
    }
}

impl SessionExtractor {
    pub fn from_id(id: Uuid, state: &State) -> Result<Self, Response> {
        let Some(session) = state.instances.get(&id) else {
            return Err(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    super::super::APPLICATION_JSON
                )
                .body(Body::from(SESSION_NOT_PRESENT))
                .unwrap())
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
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let Path((id, guild)) =
            <Path<(Uuid, NonZeroU64)> as FromRequestParts<State>>::from_request_parts(parts, state)
            .map_err(|e| Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    super::super::APPLICATION_JSON
                )
                .body(Body::from(super::player::MISSING_ID))
                .unwrap()
            )
            .await?;

        let SessionExtractor(session) = SessionExtractor::from_id(id, state)?;

        Ok(Self {
            session,
            guild
        })
    }
}
