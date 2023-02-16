use std::{future::Future, time::Duration};

use futures::FutureExt;

use crate::platform;

/// Spawns a new background task
pub fn spawn<F, T>(fut: F) -> JoinHandle<T>
where
    F: 'static + Future<Output = T>,
    T: 'static,
{
    JoinHandle {
        inner: platform::task::spawn(fut),
    }
}

pub async fn sleep(dur: Duration) {
    platform::task::sleep(dur).await
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum JoinError {
    #[error("The future was aborted")]
    Aborted,
    #[error("The future pancked")]
    #[allow(dead_code)]
    Panicked,
}

pub struct JoinHandle<T> {
    inner: platform::task::JoinHandle<T>,
}

impl<T> JoinHandle<T> {
    pub fn abort(&self) {
        self.inner.abort()
    }

    /// Returns true if the task is currently finished or aborted
    pub fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.inner.poll_unpin(cx)
    }
}
