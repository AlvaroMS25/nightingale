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
        let mut fut = self.notify.notified();
        let pinned = unsafe { Pin::new_unchecked(&mut fut) };
        pinned.enable();

        Ticket {
            inner: fut,
            queue: Some(&self.notify)
        }
    }
}

pub struct Ticket<'a> {
    inner: Notified<'a>,
    queue: Option<&'a Notify>
}

impl<'a> Future for Ticket<'a> {
    type Output = TicketPermit<'a>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let future = unsafe { Pin::new_unchecked(&mut this.inner) };
        ready!(future.poll(cx));

        Poll::Ready(TicketPermit {
            queue: this.queue.take().unwrap()
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
