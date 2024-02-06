use std::ops::Deref;
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use sysinfo::{Pid, System};
use uuid::Uuid;
use crate::api::session::Session;
use crate::search::Search;

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
    pub http: reqwest::Client,
    pub instances: DashMap<Uuid, Arc<Session>>,
    pub system: Mutex<System>,
    pub pid: Pid,
    pub search: Search
}

impl Inner {
    fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            instances: Default::default(),
            system: Mutex::new(System::new_all()),
            pid: Pid::from_u32(std::process::id()),
            search: Search::new()
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
