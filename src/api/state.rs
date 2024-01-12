use std::ops::Deref;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::api::session::Session;

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
    pub instances: DashMap<Uuid, Arc<RwLock<Session>>>
}

impl Inner {
    fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            instances: Default::default()
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
