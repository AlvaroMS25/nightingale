use std::collections::HashMap;
use thiserror::Error;
use serde_json::Value;
use crate::api::error::IntoResponseError;
use crate::source::deezer::model::ResponseError;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("Response error: {0:?}")]
    Response(ResponseError),
    #[error("Invalid deezer url provided")]
    InvalidUrl,
    #[error("No tracks found for identifier {0}")]
    NoTrackFound(usize),
    #[error("Deezer error: {0:?}")]
    Dynamic(HashMap<String, Value>)
}

impl From<ResponseError> for Error {
    fn from(value: ResponseError) -> Self {
        Self::Response(value)
    }
}
