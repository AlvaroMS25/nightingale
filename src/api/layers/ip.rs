use std::net::SocketAddr;
use std::task::{Context, Poll};
use axum::body::Body;
use axum::extract::{ConnectInfo, Request};
use axum::http::StatusCode;
use axum::RequestExt;
use axum::response::{IntoResponse, Response};
use futures_util::future::BoxFuture;
use tower::{Layer, Service, ServiceExt};
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

impl<S> Service<Request> for IpFilterLayer<S>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let mut inner = self.inner.clone().boxed_clone();
        let filter = self.filter;

        Box::pin(async move {
            let info = match req.extract_parts::<ConnectInfo<SocketAddr>>().await {
                Ok(info) => info,
                Err(e) => {
                    return Ok(e.into_response());
                }
            };

            let allowed = match info.0 {
                SocketAddr::V4(v4) => filter.v4.map(|n| n.contains(v4.ip())),
                SocketAddr::V6(v6) => filter.v6.map(|n| n.contains(v6.ip()))
            }.unwrap_or(false);

            if !allowed {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::empty())
                    .unwrap())
            }

            inner.call(req).await.map_err(From::from)
        })
    }
}
