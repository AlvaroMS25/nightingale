use tokio::sync::{Mutex, MutexGuard};
use crate::ticket::{Ticket, TicketedQueue};

pub struct TicketedMutex<T> {
    queue: TicketedQueue,
    inner: Mutex<T>
}

impl<T> TicketedMutex<T> {
    pub fn new(item: T) -> Self {
        Self {
            queue: TicketedQueue::new(),
            inner: Mutex::new(item)
        }
    }

    pub async fn lock(&self) -> MutexGuard<T> {
        self.inner.lock().await
    }

    pub fn ticket(&self) -> Entry<T> {
        Entry {
            ticket: self.queue.ticket(),
            mutex: &self.inner
        }
    }
}

pub struct Entry<'a, T> {
    ticket: Ticket<'a>,
    mutex: &'a Mutex<T>
}

impl<'a, T> Entry<'a, T> {
    pub async fn wait(self) -> MutexGuard<'a, T> {
        let _permit = self.ticket.await;
        self.mutex.lock().await
    }
}
