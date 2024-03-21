use reqwest::Client;
use songbird::input::HttpRequest;
use crate::source::{IntoResponseError, Playable, SourcePlayer};

pub struct HttpSource {
    http: Client
}

impl HttpSource {
    pub fn new(http: Client) -> Self {
        Self {
            http
        }
    }
}

#[async_trait::async_trait]
impl SourcePlayer for HttpSource {
    async fn play_url(&self, url: String) -> Result<Playable, IntoResponseError> {
        Ok(Playable {
            input: HttpRequest::new(self.http.clone(), url).into(),
            meta: Default::default()
        })
    }
}
