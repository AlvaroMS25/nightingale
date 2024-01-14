use std::sync::Arc;
use std::time::Duration;
use songbird::{Call, Event};
use tokio::sync::RwLock;
use crate::api::metrics::periodic::PeriodicMetrics;
use crate::api::session::Session;

mod periodic;
mod track;
mod driver;

#[async_trait::async_trait]
pub trait MetricsExt {
    async fn register_metrics(&mut self, session: Arc<RwLock<Session>>);
}

#[async_trait::async_trait]
impl MetricsExt for Call {
    async fn register_metrics(&mut self, session: Arc<RwLock<Session>>) {
        self.add_global_event(
            Event::Periodic(Duration::from_secs(5), None),
            PeriodicMetrics::new(Arc::clone(&session))
        )
    }
}
