use std::ops::Deref;
use std::sync::Arc;
use dashmap::DashMap;
use sysinfo::Pid;
use uuid::Uuid;
use crate::api::session::Session;
use crate::config::Config;
use crate::metrics::MetricsTracker;
use crate::ptr::SharedPtr;
use crate::source::Sources;
use crate::system::System;

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
    pub fn new(config: &Config) -> Self {
        Self(Arc::new(Inner::new(config)))
    }

    /// Returns a pointer to the underlying arc data without increasing nor decreasing the refcount.
    ///
    /// SAFETY: The caller must ensure the data is valid when they want to read from it,
    /// and must **NOT** under any circumstances, call `drop_data` on the pointer returned by this
    /// function, since that would mean the whole State instance would be deallocated.
    pub fn as_ptr(&self) -> SharedPtr<Inner> {
        unsafe {
            // we know the pointer is not null
            SharedPtr::from_ptr_unchecked(Arc::as_ptr(&self.0) as *mut _)
        }
    }
}

pub struct Inner {
    /// Http client used to make requests.
    pub http: reqwest::Client,
    /// Running session instances.
    pub instances: DashMap<Uuid, Arc<Session>>,
    /// Information about the system the server is running on.
    pub system: SharedPtr<System>,
    /// Sources supported by nightingale.
    pub sources: SharedPtr<Sources>,
}

impl Inner {
    fn new(config: &Config) -> Self {
        let http = reqwest::Client::new();
        let sys = SharedPtr::new(System::new(Pid::from_u32(std::process::id())));

        MetricsTracker::init(sys, config.metrics.clone());

        Self {
            http: http.clone(),
            instances: Default::default(),
            system: sys,
            sources: SharedPtr::new(Sources::new(http)),
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

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe { self.sources.drop_data() }
    }
}
