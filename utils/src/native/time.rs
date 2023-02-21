use derive_more::{Add, AddAssign, Sub};
/// Represents an abstract point in time
pub use std::time::Instant;
fn schedule_wakeup<F: 'static + Send + FnOnce()>(dur: Duration, callback: F) {
    tokio::spawn(async move {
        tokio::time::sleep(dur).await;
        callback()
    })
}
