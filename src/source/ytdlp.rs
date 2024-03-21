use reqwest::Client;
use songbird::input::{Compose, YoutubeDl};
use crate::source::{Playable, SourcePlayer, IntoResponseError};

pub struct Ytdlp {
    http: Client
}

impl Ytdlp {
    pub fn new(http: Client) -> Self {
        Self {
            http
        }
    }
}

#[async_trait::async_trait]
impl SourcePlayer for Ytdlp {
    async fn play_url(&self, url: String) -> Result<Playable, IntoResponseError> {
        let mut ydl = YoutubeDl::new(self.http.clone(), url);

        Ok(Playable {
            meta: ydl.aux_metadata().await?,
            input: ydl.into()
        })
    }
}
