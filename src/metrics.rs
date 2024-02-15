use prometheus_client::registry::Registry;

/// Metrics collector used in the prometheus endpoint.
#[allow(dead_code)]
pub struct MetricsTracker {
    registry: Registry
}

impl MetricsTracker {
    #[allow(dead_code)]
    pub fn new() -> Self {
        todo!()
    }
}
