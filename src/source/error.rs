use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct StringError(String);

impl<T: Error> From<T> for StringError {
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

impl Debug for StringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <String as Debug>::fmt(&self.0, f)
    }
}

impl Display for StringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <String as Display>::fmt(&self.0, f)
    }
}

impl IntoResponse for StringError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(
                axum::http::header::CONTENT_TYPE,
                "application/json"
            )
            .body(Body::from(format!(r#"{{ "message": "{self}" }}"#)))
            .unwrap()
    }
}
