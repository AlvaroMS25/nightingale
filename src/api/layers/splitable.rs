use axum::body::Body;
use axum::extract::Request;
use axum::http::request::Parts;

pub struct SplittableRequest {
    req: Option<Request>,
    body: Option<Body>
}

impl SplittableRequest {
    pub fn new(req: Request) -> Self {
        Self {
            req: Some(req),
            body: None
        }
    }

    pub fn parts(&mut self) -> Parts {
        let (parts, body) = self.req.take().expect("Request already taken").into_parts();

        self.body = Some(body);
        parts
    }

    pub fn recover(&mut self, parts: Parts) -> Request {
        assert!(self.req.is_none());
        assert!(self.body.is_some());

        Request::from_parts(parts, self.body.take().unwrap())
    }
}
