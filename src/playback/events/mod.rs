use std::sync::Arc;
use std::time::Duration;
use songbird::{Call, CoreEvent, Event, EventHandler, TrackEvent};
use tokio::sync::RwLock;
use driver::DriverMetrics;
use periodic::PeriodicMetrics;
use track::TrackMetrics;
use crate::api::session::Session;

mod periodic;
mod track;
mod driver;
pub mod resume;

#[async_trait::async_trait]
pub trait EventsExt {
    async fn register_events(&self, session: Arc<RwLock<Session>>);
}

fn chain_events<I, T, H>(call: &mut Call, events: I, handler: H)
where
    I: IntoIterator<Item = T>,
    T: Into<Event>,
    H: EventHandler + 'static + Clone
{
    for event in events {
        call.add_global_event(
            event.into(),
            handler.clone()
        );
    }
}

#[async_trait::async_trait]
impl EventsExt for Arc<RwLock<Call>> {
    async fn register_events(&self, session: Arc<RwLock<Session>>) {
        let mut call = self.write().await;
        
        call.add_global_event(
            Event::Periodic(Duration::from_secs(5), None),
            PeriodicMetrics::new(Arc::clone(&session)).await
        );

        chain_events(
            &mut call,
            [
                TrackEvent::Play,
                TrackEvent::End,
                TrackEvent::Error
            ],
            TrackMetrics::new(Arc::clone(&session)).await
        );

        chain_events(
            &mut call,
            [
                CoreEvent::DriverConnect,
                CoreEvent::DriverDisconnect,
                CoreEvent::DriverReconnect
            ],
            DriverMetrics::new(session).await
        );

        chain_events(
            &mut call,
            [
                CoreEvent::DriverConnect,
                CoreEvent::DriverDisconnect,
                CoreEvent::DriverReconnect
            ],
            DebugEvents
        );

        call.add_global_event(
            songbird::CoreEvent::DriverConnect.into(),
            resume::ResumeOnMove::new(Arc::clone(&self))
        );
    }
}

#[derive(Clone)]
pub struct DebugEvents;

#[async_trait::async_trait]
impl EventHandler for DebugEvents {
    #[tracing::instrument(skip(self, ctx))]
    async fn act(&self, ctx: &songbird::EventContext<'_>) -> Option<Event> {
        tracing::info!("Debug event: {ctx:?}");
        None
    }
}
