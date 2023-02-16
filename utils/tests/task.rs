use futures::FutureExt;
use std::time::Duration;

use utils::task::{sleep, spawn, JoinError};
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
#[cfg(target_arch = "wasm32")]
async fn tasks() {
    let a = spawn(async move {
        loop {
            sleep(Duration::from_millis(100)).await;
        }
    });

    let b = spawn(async {
        sleep(Duration::from_millis(100)).await;
        "Hello, World".to_string()
    });

    let c = spawn(async {
        sleep(Duration::from_millis(500)).await;
        "Hello, World".to_string()
    });

    assert_eq!(c.await, Ok("Hello, World".to_string()));

    // At this point, b is finished too
    assert!(b.is_finished());

    assert!(!a.is_finished());

    a.abort();

    assert_eq!(b.now_or_never(), Some(Ok("Hello, World".to_string())));

    sleep(Duration::from_millis(100)).await;

    assert!(a.is_finished());
    assert_eq!(a.await, Err(JoinError::Aborted));
}
