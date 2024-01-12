use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};

#[derive(Clone)]
pub struct Abort(Arc<Inner>);

impl Abort {
    pub fn new() -> Self {
        Self(Arc::new(Inner {
            complete: AtomicBool::new(false),
            waker: Mutex::new(None)
        }))
    }

    pub fn as_future(&self) -> AbortFuture {
        AbortFuture(Arc::clone(&self.0))
    }

    pub fn abort(&self) {
        self.0.complete.store(true, Ordering::Release);

        self.0.with_lock(|mut lock| {
            lock.take().map(|w| w.wake());
        })
    }
}

pub struct Inner {
    complete: AtomicBool,
    waker: Mutex<Option<Waker>>
}

impl Inner {
    pub fn with_lock<F, R>(&self, fun: F) -> R
    where
        F: for<'l> FnOnce(MutexGuard<'l, Option<Waker>>) -> R
    {
        let lock = self.waker.lock()
            .unwrap_or_else(|l| l.into_inner());

        fun(lock)
    }
}

pub struct AbortFuture(Arc<Inner>);

impl Future for AbortFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0.complete.load(Ordering::Acquire) {
            Poll::Ready(())
        } else {
            let waker = cx.waker().clone();
            self.0.with_lock(move |mut lock| {
                *lock = Some(waker);
            });
            Poll::Pending
        }
    }
}
