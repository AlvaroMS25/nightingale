use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::futures::Notified;
use tokio::sync::Notify;

pub struct TicketedQueue {
    notify: Notify
}

impl TicketedQueue {
    pub fn new() -> Self {
        let notify = Notify::new();
        notify.notify_one(); // First ticket must proceed instantly

        Self {
            notify
        }
    }

    pub fn ticket(&self) -> Ticket {
        let mut fut = self.notify.notified();
        let pinned = unsafe { Pin::new_unchecked(&mut fut) };
        pinned.enable();

        Ticket {
            inner: fut,
            queue: &self.notify
        }
    }
}

pub struct Ticket<'a> {
    inner: Notified<'a>,
    queue: &'a Notify
}

impl<'a> Future for Ticket<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let future = unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner) };
        future.poll(cx)
    }
}

impl Drop for Ticket<'_> {
    fn drop(&mut self) {
        self.queue.notify_one();
    }
}
