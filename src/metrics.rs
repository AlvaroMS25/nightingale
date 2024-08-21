use std::mem::MaybeUninit;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::time::Duration;
use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::response::Response;
use parking_lot::Mutex;
use prometheus_client::metrics::gauge::Gauge as G;
use prometheus_client::metrics::info::Info;
use prometheus_client::registry::Registry;
use tokio::task::JoinHandle;
//use prometheus::{Encoder, Gauge, IntGauge, Registry};
use crate::api::error::IntoResponseError;
use crate::config::MetricsOptions;
use crate::ptr::SharedPtr;
use crate::system::System;

type IntGauge = G<u32, AtomicU32>;
type FloatGauge = G<f64, AtomicU64>;

static mut METRICS: MaybeUninit<MetricsTracker> = MaybeUninit::uninit();

pub fn metrics() -> &'static MetricsTracker {
    unsafe {
        METRICS.assume_init_ref()
    }
}

pub unsafe fn drop_metrics() {
    if let Some(t) = metrics().task.lock().take() {
        t.abort();
    }

    METRICS.assume_init_drop();
}

/// Metrics collector used in the prometheus endpoint.
pub struct MetricsTracker {
    pub system: SharedPtr<System>,
    pub options: MetricsOptions,
    pub registry: Arc<Registry>,
    pub sessions: IntGauge,
    pub playing_players: IntGauge,
    pub active_players: IntGauge,
    pub cpu_usage: FloatGauge,
    pub total_cpu_usage: FloatGauge,
    pub ram_usage: FloatGauge,
    pub virtual_ram_usage: FloatGauge,
    pub task: Mutex<Option<JoinHandle<()>>>
}

impl MetricsTracker {
    #[allow(dead_code)]
    pub fn init(system: SharedPtr<System>, opts: MetricsOptions) {
        let mut registry = Registry::default();

        let sessions = IntGauge::default();
        let playing_players = IntGauge::default();
        let active_players = IntGauge::default();
        let cpu_usage = FloatGauge::default();
        let total_cpu_usage = FloatGauge::default();
        let ram_usage = FloatGauge::default();
        let virtual_ram_usage = FloatGauge::default();

        registry.register("Sessions", "Number of active sessions", sessions.clone());
        registry.register("PlayingPlayers", "Players that are currently playing audio", playing_players.clone());
        registry.register("ActivePlayers", "Number of active players", active_players.clone());
        registry.register("ServerCPUUsage", "Server percentage of CPU usage", cpu_usage.clone());
        registry.register("TotalCPUUsage", "System total percentage of CPU usage", total_cpu_usage.clone());
        registry.register("RSSRAMUsage", "Server RAM usage in MB", ram_usage.clone());
        registry.register("VirtualRAMUsage", "Server Virtual RAM usage in MB", virtual_ram_usage.clone());

        let this = Self {
            system,
            options: opts,
            registry: Arc::new(registry),
            sessions,
            playing_players,
            active_players,
            cpu_usage,
            total_cpu_usage,
            ram_usage,
            virtual_ram_usage,
            task: Default::default(),
        };

        unsafe {
            METRICS.write(this);
        }

        metrics().start_task();
    }

    pub fn build_response(&self) -> Result<Response, IntoResponseError> {
        let mut buf = String::new();
        prometheus_client::encoding::text::encode(&mut buf, &self.registry)?;

        Response::builder()
            .header(CONTENT_TYPE, "text/plain; version=0.0.4")
            .body(Body::from(buf))
            .map_err(From::from)
    }

    fn start_task(&'static self) {
        let handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(self.options.update_seconds)).await;

                let info = self.system.update_get();

                self.cpu_usage.set(info.cpu.process_usage as _);
                self.total_cpu_usage.set(info.cpu.total_usage as _);
                self.ram_usage.set((info.memory.memory / 1024 / 1024) as _);
                self.virtual_ram_usage.set((info.memory.virtual_memory / 1024 / 1024) as _);
            }
        });

        *self.task.lock() = Some(handle);
    }
}
