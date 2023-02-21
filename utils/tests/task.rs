use futures::FutureExt;
use std::time::Duration;

use utils::{
    task::{spawn_local, JoinError},
    timer::{sleep, TimerWheel},
};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn tasks() {
    let a = spawn_local(|| async move {
        loop {
            gloo::timers::future::sleep(Duration::from_millis(100)).await;
        }
    });

    let b = spawn_local(|| async {
        gloo::timers::future::sleep(Duration::from_millis(100)).await;
        "Hello, World".to_string()
    });

    let c = spawn_local(|| async {
        gloo::timers::future::sleep(Duration::from_millis(500)).await;
        "Hello, World".to_string()
    });

    assert_eq!(c.await, Ok("Hello, World".to_string()));

    // At this point, b is finished too
    assert!(b.is_finished());

    assert!(!a.is_finished());

    a.abort();

    assert_eq!(b.now_or_never(), Some(Ok("Hello, World".to_string())));

    gloo::timers::future::sleep(Duration::from_millis(100)).await;

    assert!(a.is_finished());
    assert_eq!(a.await, Err(JoinError::Aborted));
}
