use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};
use std::fmt;
use parking_lot::{Mutex, MutexGuard};

#[derive(Clone)]
pub struct Abort(Arc<Inner>);

impl Abort {
    pub fn new() -> Self {
        Self(Arc::new(Inner {
            complete: AtomicBool::new(false),
            waker: Mutex::new(None)
        }))
    }

    /// Creates a future that will resolve when [`Abort::abort`](Self::abort)
    /// is called.
    pub fn as_future(&self) -> AbortFuture {
        AbortFuture(Arc::clone(&self.0))
    }

    /// Aborts any [`AbortFuture`]s created by [`Abort::as_future`].
    /// Subsequent futures created by [`Abort::as_future`] will resolve immediately.
    ///
    /// [`Abort::as_future`]: Self::as_future
    pub fn abort(&self) {
        self.0.complete.store(true, Ordering::Release);

        self.0.with_lock(|mut lock| {
            if let Some(w) = lock.take() {
                w.wake();
            }
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
        let lock = self.waker.lock();

        fun(lock)
    }
}

/// Future created by [`Abort#as_future`](Abort::as_future), resolves when
/// [`Abort#abort`](Abort::abort) is called.
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

impl fmt::Debug for Abort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Abort").finish()
    }
}
