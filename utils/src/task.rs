use std::{future::Future, time::Duration};

use futures::FutureExt;

use crate::platform;

/// Spawns a new background task
pub fn spawn<F, T>(fut: F) -> JoinHandle<T>
where
    F: 'static + Future<Output = T>,
    T: 'static,
{
    JoinHandle(platform::task::spawn(fut))
}

pub async fn sleep(dur: Duration) {
    platform::task::sleep(dur).await
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum JoinError {
    #[error("The future was aborted")]
    Aborted,
    #[error("The future panicked")]
    #[allow(dead_code)]
    Panicked,
}

pub struct JoinHandle<T>(platform::task::JoinHandle<T>);

#[cfg(not(target_arch = "wasm32"))]
impl From<tokio::task::JoinHandle> for JoinHandle<T> {
    fn from(value: tokio::task::JoinHandle) -> Self {
        Self(value)
    }
}

impl<T> JoinHandle<T> {
    pub fn abort(&self) {
        self.0.abort()
    }

    /// Returns true if the task is currently finished or aborted
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.0.poll_unpin(cx)
    }
}

pub struct AbortOnDrop<T>(JoinHandle<T>);

impl<T> Future for AbortOnDrop<T> {
    type Output = Result<T, JoinError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.0.poll_unpin(cx)
    }
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}
