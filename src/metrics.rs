use std::sync::atomic::AtomicU32;
use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::response::Response;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::metrics::info::Info;
use prometheus_client::registry::Registry;
use sysinfo::System;
use crate::api::error::IntoResponseError;
use crate::config::MetricsOptions;

pub type GaugeU32 = Gauge<u32, AtomicU32>;

/// Metrics collector used in the prometheus endpoint.
pub struct MetricsTracker {
    pub registry: Registry,
    pub sessions: GaugeU32,
    pub playing_players: GaugeU32,
    pub active_players: GaugeU32,
    pub cpu_usage: GaugeU32,
    pub ram_usage: GaugeU32,
    pub playing_players_histogram: Histogram,
    pub active_players_histogram: Histogram,
    pub cpu_usage_histogram: Histogram,
    pub ram_usage_histogram: Histogram,
}

fn gen_buckets(minutes: u16) -> impl Iterator<Item = f64> + Clone {
    let minutes = -(minutes as i16);

    (minutes ..= 0)
        .map(|v| v as f64)
}

impl MetricsTracker {
    #[allow(dead_code)]
    pub fn new(config: &MetricsOptions) -> Self {
        let mut registry = Registry::with_prefix("Nightingale metrics");
        let buckets = gen_buckets(config.retain_minutes);

        let info = Info::new(Self::os_info());

        let sessions = GaugeU32::default();

        let playing_players_gauge = GaugeU32::default();
        let playing_players_histogram = Histogram::new(buckets.clone());

        let active_players_gauge = GaugeU32::default();
        let active_players_histogram = Histogram::new(buckets.clone());

        let cpu_usage_gauge = GaugeU32::default();
        let cpu_usage_histogram = Histogram::new(buckets.clone());

        let ram_usage_gauge = GaugeU32::default();
        let ram_usage_histogram = Histogram::new(buckets);

        registry.register("System information", "Information about the system the server runs on", info);
        registry.register("Sessions", "Number of active sessions", sessions.clone());
        registry.register("Playing players", "Players that are currently playing audio", playing_players_gauge.clone());
        registry.register("Active players", "Number of active players", active_players_gauge.clone());
        registry.register("CPU usage", "Server CPU usage", cpu_usage_gauge.clone());
        registry.register("RAM usage", "Server RAM usage", ram_usage_gauge.clone());

        registry.register("Playing players histogram", "Histogram of playing players", playing_players_histogram.clone());
        registry.register("Active players histogram", "Histogram of active players", active_players_histogram.clone());
        registry.register("CPU usage histogram", "Histogram of CPU usage", cpu_usage_histogram.clone());
        registry.register("RAM usage histogram", "Histogram of RAM usage", ram_usage_histogram.clone());

        Self {
            registry,
            sessions,
            playing_players: playing_players_gauge,
            active_players: active_players_gauge,
            cpu_usage: cpu_usage_gauge,
            ram_usage: ram_usage_gauge,
            playing_players_histogram,
            active_players_histogram,
            cpu_usage_histogram,
            ram_usage_histogram
        }
    }

    fn os_info() -> Vec<(String, String)> {
        let sys = System::new();

        let cpu = sys.global_cpu_info();
        let os_name = String::from(std::env::consts::OS);

        let cores = if let Some(c) = sys.physical_core_count() {
            c.to_string()
        } else {
            String::from("Unknown")
        };
        let mb = sys.total_memory() / 1024 / 1024;

        vec![
            (String::from("OS"), os_name),
            (String::from("CPU"), String::from(cpu.name())),
            (String::from("CPU Cores"), cores),
            (String::from("RAM (MB)"), mb.to_string())
        ]
    }

    pub fn build_response(&self) -> Result<Response, IntoResponseError> {
        let mut buf = String::new();
        prometheus_client::encoding::text::encode(&mut buf, &self.registry)?;

        Response::builder()
            .header(CONTENT_TYPE, "application/openmetrics-text; version=1.0.0; charset=utf-8")
            .body(Body::from(buf))
            .map_err(From::from)
    }
}
