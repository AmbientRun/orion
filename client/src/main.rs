// use std::time::Duration;

// use chrono::Utc;

use std::time::Duration;

use tokio::{runtime, time::sleep};

fn main() {
    println!("Hello, World!");
    let now = chrono::Utc::now();
    println!("The time is now: {now:?}");
    let rt = runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(async move {
        eprintln!("Sleeping");
        sleep(Duration::from_millis(500)).await;
        eprintln!("Finished sleeping")
    });

    // panic!("Is this good enough for you?");
    // let dur = Duration::from_millis(500);
    // loop {
    // std::thread::sleep(std::time::Duration::from_millis(500));
    // }
    // with_tokio_runtime(run()).await
}
