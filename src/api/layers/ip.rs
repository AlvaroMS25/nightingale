use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::extract::{ConnectInfo, FromRequestParts, Request};
use axum::http::StatusCode;
use axum::RequestExt;
use axum::response::Response;
use futures::ready;
use futures_util::FutureExt;
use tower::{Layer, Service};

use crate::api::layers::splitable::SplittableRequest;
use crate::config::FilterIps;

/// Layer that filters connections using the IPs provided in the configuration file,
/// filters IPV4 or/and IPV6 IPs.
#[derive(Clone)]
pub struct IpFilter(pub FilterIps);

impl<S> Layer<S> for IpFilter {
    type Service = IpFilterLayer<S>;

    fn layer(&self, inner: S) -> Self::Service {
        IpFilterLayer {
            filter: self.0,
            inner
        }
    }
}

/// Inner [`IpFilter`] service.
#[derive(Clone)]
pub struct IpFilterLayer<S> {
    filter: FilterIps,
    inner: S
}

pub struct IpFilterFuture<F, E> {
    allowed: bool,
    fut: Option<F>,
    marker: PhantomData<fn() -> Result<Response, E>>
}

impl<F, E> Future for IpFilterFuture<F, E>
where
    F: Future<Output = Result<Response, E>> + Send + Unpin + 'static,
    E: 'static
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.allowed {
            Poll::Ready(Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::empty())
                .unwrap()))
        } else {
            //Pin::new(&mut self.fut.as_mut().unwrap()).poll(cx)
            let Some(fut) = self.fut.as_mut() else {
                panic!("Future polled after completion");
            };

            let res = ready!(Pin::new(fut).poll(cx));
            self.fut.take();

            Poll::Ready(res)
        }
    }
}

impl<S> Service<Request> for IpFilterLayer<S>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + Unpin + 'static
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = IpFilterFuture<S::Future, S::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut splitable = SplittableRequest::new(req);
        let mut parts = splitable.parts();

        let info = ConnectInfo::<SocketAddr>::from_request_parts(&mut parts, &())
            // ConnectInfo uses Extension as its extractor, and Extension itself does not
            // await anything, so we are sure the future completes at first poll.
            .now_or_never().unwrap()
            // We tell axum server to provide us with the connection information, so this cannot fail.
            .unwrap();

        let req = splitable.recover(parts);

        let allowed = match info.0 {
            SocketAddr::V4(v4) => self.filter.v4.map(|n| n.contains(v4.ip())),
            SocketAddr::V6(v6) => self.filter.v6.map(|n| n.contains(v6.ip()))
        }.unwrap_or(false);

        IpFilterFuture {
            allowed,
            fut: if allowed { Some(self.inner.call(req)) } else { None },
            marker: PhantomData
        }
    }
}
