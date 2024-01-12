use axum::body::Bytes;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum PlaySource {
    Link(String),
    Bytes(Vec<u8>)
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlayOptions {
    pub force_play: bool,
    pub source: PlaySource
}
