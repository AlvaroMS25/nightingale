use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum Error {
    Io(#[from] io::Error),
    Json(#[from] serde_json::Error),
    Yaml(#[from] serde_yaml::Error)
}