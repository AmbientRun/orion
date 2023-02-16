use std::{future::Future, time::Duration};
pub use tokio::JoinHandle;

/// Spawns a new background task
pub fn spawn<F, T>(fut: F) -> JoinHandle<T>
where
    F: 'static + Future<Output = T>,
    T: 'static,
{
    tokio::spawn(fut)
}

#[inline]
pub async fn sleep(dur: Duration) {
    tokio::time::sleep(dur).await
}
