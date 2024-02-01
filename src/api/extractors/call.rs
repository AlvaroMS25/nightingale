use std::num::NonZeroU64;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Response;
use serde::Deserialize;
use songbird::Call;
use tokio::sync::RwLock;
use crate::api::extractors::session::SessionExtractor;
use crate::api::state::State;

const NOT_CONNECTED: &str = r#"{"message": "Not connected to voice"}"#;
pub const MISSING_GUILD_ID: &str = r#"{"message": "Missing guild ID"}"#;

pub struct CallExtractor(pub Arc<RwLock<Call>>);

#[async_trait::async_trait]
impl FromRequestParts<State> for CallExtractor {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        #[derive(Deserialize)]
        struct GuildQuery {
            guild_id: NonZeroU64
        }

        let SessionExtractor(session) = SessionExtractor::from_request_parts(parts, state).await?;
        let Query(query) = Query::<GuildQuery>::from_request_parts(parts, state).await
            .map_err(|_| {
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(
                        axum::http::header::CONTENT_TYPE,
                        super::super::APPLICATION_JSON
                    )
                    .body(Body::from(MISSING_GUILD_ID))
                    .unwrap()
            })?;

        let lock = session.read().await;

        let Some(call) = lock.playback.get_call(query.guild_id) else {
            return Err(
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(
                        axum::http::header::CONTENT_TYPE,
                        super::super::APPLICATION_JSON
                    )
                    .body(Body::from(NOT_CONNECTED))
                    .unwrap()
            );
        };

        Ok(CallExtractor(call))
    }
}
