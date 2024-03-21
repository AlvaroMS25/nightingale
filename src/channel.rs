use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use crate::api::model::gateway::Outgoing;

pub struct Sender(mpsc::UnboundedSender<Outgoing>);
pub struct Receiver(mpsc::UnboundedReceiver<Outgoing>);

impl Sender {
    pub fn send(&self, value: Outgoing) -> Result<(), SendError<Box<Outgoing>>> {
        self.0.send(value)
            .map_err(|e| SendError(Box::new(e.0)))
    }
}

pub fn new() -> (Sender, Receiver) {
    let (tx, rx) = mpsc::unbounded_channel();

    (
        Sender(tx),
        Receiver(rx)
    )
}

impl Clone for Sender {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Stream for Receiver {
    type Item = Outgoing;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().0.poll_recv(cx)
    }
}
