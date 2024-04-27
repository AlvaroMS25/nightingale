use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Error type that implements [`IntoResponse`], so it can be used from within
/// api routes directly.
pub struct IntoResponseError {
    pub msg: String,
    pub status: StatusCode
}

impl IntoResponseError {
    pub fn new(msg: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }
}

impl<T: Error> From<T> for IntoResponseError {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl Debug for IntoResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <String as Debug>::fmt(&self.msg, f)
    }
}

impl Display for IntoResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <String as Display>::fmt(&self.msg, f)
    }
}

impl IntoResponse for IntoResponseError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(self.status)
            .header(
                axum::http::header::CONTENT_TYPE,
                "application/json"
            )
            .body(Body::from(format!(r#"{{ "message": "{self}" }}"#)))
            .unwrap()
    }
}
