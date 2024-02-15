use std::sync::Arc;
use std::time::Duration;
use songbird::{Call, CoreEvent, Event, EventHandler, TrackEvent};
use driver::DriverEvents;
use periodic::PeriodicEvents;
use track::TrackEvents;
use crate::api::session::Session;

mod periodic;
mod track;
mod driver;

#[async_trait::async_trait]
pub trait EventsExt {
    async fn register_events(&mut self, session: Arc<Session>);
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
impl EventsExt for Call {
    async fn register_events(&mut self, session: Arc<Session>) {
        self.add_global_event(
            Event::Periodic(Duration::from_secs(5), None),
            PeriodicEvents::new(Arc::clone(&session)).await
        );

        chain_events(
            self,
            [
                TrackEvent::Play,
                TrackEvent::End,
                TrackEvent::Error
            ],
            TrackEvents::new(Arc::clone(&session)).await
        );

        chain_events(
            self,
            [
                CoreEvent::DriverConnect,
                CoreEvent::DriverDisconnect,
                CoreEvent::DriverReconnect
            ],
            DriverEvents::new(session).await
        );

        chain_events(
            self,
            [
                CoreEvent::DriverConnect,
                CoreEvent::DriverDisconnect,
                CoreEvent::DriverReconnect
            ],
            DebugEvents
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
