use std::sync::Arc;
use axum::body::Body;
use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Response;
use serde::Deserialize;
use uuid::Uuid;
use crate::api::session::Session;
use crate::api::state::State;

pub const MISSING_SESSION_ID: &str = r#"{"message": "Missing session ID"}"#;
pub const SESSION_NOT_PRESENT: &str = r#"{"message": "Session not present"}"#;

/// Extractor that takes a session from the url query and resolves to the corresponding session,
/// if the session is not provided or invalid, it returns a 400 Bad request with the corresponding
/// error message.
pub struct SessionExtractor(pub Arc<Session>);

#[async_trait::async_trait]
impl FromRequestParts<State> for SessionExtractor {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        #[derive(Deserialize)]
        struct SessionQuery {
            session: Uuid
        }

        let Query(query) = Query::<SessionQuery>::from_request_parts(parts, state).await
            .map_err(|_| {
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(
                        axum::http::header::CONTENT_TYPE,
                        super::super::APPLICATION_JSON
                    )
                    .body(Body::from(MISSING_SESSION_ID))
                    .unwrap()
            })?;

        let Some(session) = state.instances.get(&query.session) else {
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
