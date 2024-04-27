use std::ops::Deref;
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use sysinfo::{Pid, System};
use uuid::Uuid;
use crate::api::session::Session;
use crate::source::Sources;

/// The state shared throughout requests.
#[derive(Clone)]
pub struct State(Arc<Inner>);

impl Deref for State {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl State {
    pub fn new() -> Self {
        Self(Arc::new(Inner::new()))
    }
}

pub struct Inner {
    /// Http client used to make requests.
    pub http: reqwest::Client,
    /// Running session instances.
    pub instances: DashMap<Uuid, Arc<Session>>,
    /// Information about the system the server is running on.
    pub system: Mutex<System>,
    /// The Pid of the server.
    pub pid: Pid,
    /// Sources supported by nightingale.
    pub sources: Arc<Sources>
}

impl Inner {
    fn new() -> Self {
        let http = reqwest::Client::new();
        Self {
            http: http.clone(),
            instances: Default::default(),
            system: Mutex::new(System::new_all()),
            pid: Pid::from_u32(std::process::id()),
            sources: Arc::new(Sources::new(http))
        }
    }

    pub fn generate_uuid(&self) -> Uuid {
        loop {
            let candidate = Uuid::new_v4();

            if !self.instances.contains_key(&candidate) {
                return candidate;
            }
        }
    }
}
