use std::sync::Arc;
use songbird::{Call, Event, EventContext, EventHandler};
use tokio::sync::Mutex;

pub struct ResumeOnMove {
    call: Arc<Mutex<Call>>
}

#[async_trait::async_trait]
impl EventHandler for ResumeOnMove {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        None
    }
}
