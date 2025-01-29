use std::path::PathBuf;
use ipnet::{Ipv4Net, Ipv6Net};
use serde::Deserialize;
use tracing::Level;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub server: Server,
    #[serde(default)]
    pub logging: LoggingOptions,
    pub metrics: MetricsOptions,
    #[serde(default)]
    pub loki: Option<LokiOptions>
}

#[derive(Deserialize, Debug)]
pub struct Server {
    #[cfg(not(feature = "docker"))]
    pub address: String,
    #[cfg(not(feature = "docker"))]
    pub port: u16,
    pub password: String,
    pub ssl: Option<SslOptions>,
    pub filter_ips: Option<FilterIps>
}

impl Server {
    #[cfg(feature = "docker")]
    pub fn port(&self) -> u16 {
        8081
    }

    #[cfg(not(feature = "docker"))]
    pub fn port(&self) -> u16 {
        self.port
    }

    #[cfg(feature = "docker")]
    pub fn address(&self) -> &str {
        "0.0.0.0"
    }

    #[cfg(not(feature = "docker"))]
    pub fn address(&self) -> &str {
        &self.address
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct SslOptions {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct LoggingOptions {
    pub enable: bool,
    #[serde(default)]
    pub level: LoggingLevel,
    #[serde(default)]
    pub output: LoggingOutput,
    #[serde(default)]
    pub file: Option<String>
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct FilterIps {
    pub v4: Option<Ipv4Net>,
    pub v6: Option<Ipv6Net>
}

impl Default for LoggingOptions {
    fn default() -> Self {
        Self {
            enable: true,
            level: Default::default(),
            output: Default::default(),
            file: None
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

impl From<LoggingLevel> for Level {
    fn from(value: LoggingLevel) -> Self {
        match value {
            LoggingLevel::Error => Level::ERROR,
            LoggingLevel::Warn => Level::WARN,
            LoggingLevel::Info => Level::INFO,
            LoggingLevel::Debug => Level::DEBUG,
            LoggingLevel::Trace => Level::TRACE
        }
    }
}

#[derive(Deserialize, Debug, Copy, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum LoggingOutput {
    #[default]
    StdOut,
    File
}

#[derive(Deserialize, Debug, Clone)]
pub struct MetricsOptions {
    pub update_seconds: u64,
    pub enable_loki: bool
}

#[derive(Deserialize, Debug, Clone)]
pub struct LokiOptions {
    pub url: String,
    pub user: String,
    pub password: String
}
