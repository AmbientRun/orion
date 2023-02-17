use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Poll, Waker},
    time::Duration,
};

use futures::{
    future::abortable,
    stream::{AbortHandle, Abortable},
};
use parking_lot::Mutex;
use pin_project::{pin_project, pinned_drop};

use crate::task::JoinError;
/// Spawns a new background task
pub fn spawn<F, T>(fut: F) -> JoinHandle<T>
where
    F: 'static + Future<Output = T>,
    T: 'static,
{
    let state = Arc::new(InnerState::new());

    let (fut, abort) = abortable(fut);
    let fut = WrappedFuture {
        fut,
        state: state.clone(),
    };

    wasm_bindgen_futures::spawn_local(fut);

    JoinHandle { state, abort }
}

struct InnerState<T> {
    waker: Mutex<Option<Waker>>,
    res: Mutex<Option<Result<T, JoinError>>>,
    woken: AtomicBool,
}

impl<T> InnerState<T> {
    fn new() -> Self {
        Self {
            waker: Mutex::new(None),
            res: Mutex::new(None),
            woken: AtomicBool::new(false),
        }
    }

    fn wake(&self) {
        // Set woken regardless of waker, since the task can complete before the JoinHandle is
        // polled
        self.woken.store(true, Ordering::SeqCst);

        if let Some(waker) = &mut *self.waker.lock() {
            waker.wake_by_ref();
        }
    }
}

#[pin_project(PinnedDrop)]
struct WrappedFuture<F, T> {
    #[pin]
    fut: Abortable<F>,
    state: Arc<InnerState<T>>,
}

#[pinned_drop]
impl<F, T> PinnedDrop for WrappedFuture<F, T> {
    fn drop(self: Pin<&mut Self>) {
        let mut res = self.state.res.lock();
        if res.is_none() {
            // Cancelled on behalf of the executor
            *res = Some(Err(JoinError::Aborted));

            self.state.wake();
        }
    }
}

impl<F, T> Future for WrappedFuture<F, T>
where
    F: Future<Output = T>,
{
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let p = self.project();

        if let Poll::Ready(value) = p.fut.poll(cx) {
            let mut res = p.state.res.lock();
            assert!(res.is_none(), "Future completed twice");
            *res = Some(value.map_err(|_| JoinError::Aborted));

            p.state.wake();

            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub struct JoinHandle<T> {
    state: Arc<InnerState<T>>,
    abort: AbortHandle,
}

impl<T> JoinHandle<T> {
    pub fn abort(&self) {
        self.abort.abort()
    }

    /// Returns true if the task is currently finished or aborted
    pub fn is_finished(&self) -> bool {
        self.state.res.lock().is_some()
    }
}

#[inline]
pub async fn sleep(dur: Duration) {
    gloo::timers::future::sleep(dur).await
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self
            .state
            .woken
            .compare_exchange(true, false, Ordering::Release, Ordering::Relaxed)
            .is_ok()
        {
            eprintln!("Future completed");

            let value = self.state.res.lock().take().unwrap();
            Poll::Ready(value)
        } else {
            // Set a waker
            *self.state.waker.lock() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[derive(Clone)]
pub struct RuntimeHandle {}

impl RuntimeHandle {
    pub fn spawn<F, T>(&self, fut: F) -> JoinHandle<T>
    where
        F: 'static + Future<Output = T>,
        T: 'static,
    {
        spawn(fut)
    }
}
