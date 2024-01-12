use std::path::PathBuf;
use serde::Deserialize;
use tracing::Level;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub server: Server,
    #[serde(default)]
    pub logging: LoggingOptions
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub address: String,
    pub port: u16,
    #[serde(default)]
    pub http2: bool,
    pub ssl: Option<SslOptions>
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct SslOptions {
    pub enable: bool,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub use_openssl: bool
}

#[derive(Deserialize, Debug)]
pub struct LoggingOptions {
    pub enable: bool,
    pub level: LoggingLevel
}

impl Default for LoggingOptions {
    fn default() -> Self {
        Self {
            enable: true,
            level: Default::default()
        }
    }
}

#[derive(Deserialize, Debug, Copy, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace
}

impl Into<Level> for LoggingLevel {
    fn into(self) -> Level {
        match self {
            Self::Error => Level::ERROR,
            Self::Warn => Level::WARN,
            Self::Info => Level::INFO,
            Self::Debug => Level::DEBUG,
            Self::Trace => Level::TRACE
        }
    }
}
