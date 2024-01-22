use std::sync::Arc;
use songbird::{input::Input, tracks::Track, Call, Event, EventContext, EventHandler};
use tokio::sync::RwLock;
use tracing::info;

use crate::playback::mock::MockMediaSource;

pub struct ResumeOnMove {
    call: Arc<RwLock<Call>>
}

impl ResumeOnMove {
    pub fn new(call: Arc<RwLock<Call>>) -> Self {
        Self {
            call
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for ResumeOnMove {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let EventContext::DriverConnect(_) = ctx else { return None; };

        let call = self.call.read().await;

        if call.queue().current().is_some() {
            drop(call);
            info!("Resuming after move");
            //let _ = current.play();
            //call.play(current)
            let t = Track::new(Input::Lazy(Box::new(MockMediaSource{})));
            self.call.write().await.play(t.pause());
        }

        None
    }
}
