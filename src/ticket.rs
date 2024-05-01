use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, ready};
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
        let mut fut = Box::pin(self.notify.notified());
        fut.as_mut().enable();

        Ticket {
            inner: fut,
            queue: Some(&self.notify)
        }
    }
}

pub struct Ticket<'a> {
    inner: Pin<Box<Notified<'a>>>,
    queue: Option<&'a Notify>
}

impl<'a> Future for Ticket<'a> {
    type Output = TicketPermit<'a>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let future = self.inner.as_mut();
        ready!(future.poll(cx));

        Poll::Ready(TicketPermit {
            queue: self.queue.take().unwrap()
        })
    }
}

impl Drop for Ticket<'_> {
    fn drop(&mut self) {
        if let Some(queue) = self.queue.take() {
            queue.notify_one();
        }
    }
}

pub struct TicketPermit<'a> {
    queue: &'a Notify
}

impl Drop for TicketPermit<'_> {
    fn drop(&mut self) {
        self.queue.notify_one();
    }
}
