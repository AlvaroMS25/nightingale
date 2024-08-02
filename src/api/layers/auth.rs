use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::process::Output;
use std::task::{Context, Poll};
use axum::{body::Body, extract::Request, http::{StatusCode, header}, response::Response};
use futures::ready;
use futures_util::future::BoxFuture;
use tower::{Layer, Service};
use tracing::warn;

/// Authentication layer that verifies the password is provided on a per-request basis
/// and denies requests that don't have it or provide an incorrect one.
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

/// Inner [`RequireAuth`] service.
#[derive(Clone)]
pub struct RequireAuthService<S> {
    password: String,
    inner: S
}

pub struct RequireAuthFuture<F, E> {
    correct: bool,
    fut: Option<F>,
    marker: PhantomData<fn() -> Result<Response, E>>
}

impl<F, E> Future for RequireAuthFuture<F, E>
where
    F: Future<Output = Result<Response, E>> + Send + Unpin + 'static,
    E: 'static
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.correct {
            warn!("Incorrect or no authorization provided");
            return Poll::Ready(Ok(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header(
                    header::WWW_AUTHENTICATE,
                    r#"Basic realm="Nightingale server", charset="UTF-8""#
                )
                .body(Body::empty())
                .unwrap()))
        }

        let Some(fut) = self.fut.as_mut() else {
            panic!("Future polled after completion");
        };

        let res = ready!(Pin::new(fut).poll(cx));
        self.fut.take();

        Poll::Ready(res)
    }
}

impl<S> Service<Request> for RequireAuthService<S> 
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + Unpin + 'static
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = RequireAuthFuture<S::Future, S::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let auth = req.headers()
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok());

        let correct = match auth {
            Some(a) if a == self.password => true,
            _ => false
        };

        RequireAuthFuture {
            correct,
            fut: if correct { Some(self.inner.call(req)) } else { None },
            marker: PhantomData,
        }
    }
}
