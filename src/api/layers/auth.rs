use axum::{body::Body, extract::Request, http::{StatusCode, header}, response::Response};
use futures_util::future::BoxFuture;
use tower::{Layer, Service};
use tracing::warn;

use super::state::State;

#[derive(Clone)]
pub struct RequireAuth(pub String);

impl<S> Layer<S> for RequireAuth {
    type Service = RequireAuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireAuthService {
            password: self.0.clone(),
            inner
        }
    }
}

#[derive(Clone)]
pub struct RequireAuthService<S> {
    password: String,
    inner: S
}

impl<S> Service<Request> for RequireAuthService<S> 
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let auth = req.headers()
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok());

        match auth {
            Some(a) if a == self.password => {},
            _ => return Box::pin(async {
                warn!("Incorrect or no authorization provided");
                Ok(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header(
                    header::WWW_AUTHENTICATE,
                    r#"Basic realm="Nightingale server", charset="UTF-8""#
                )
                .body(Body::empty())
                .unwrap())
            })
        };

        let fut = self.inner.call(req);
        Box::pin(async move {
            fut.await.map_err(From::from)
        })
    }
}
