use std::time::Duration;

use chrono::Utc;

fn main() {
    println!("Hello, World!");
    let now = Utc::now();
    println!("The time is now: {now:?}");
    // let dur = Duration::from_millis(500);
    // loop {
    // std::thread::sleep(std::time::Duration::from_millis(500));
    // }
    // with_tokio_runtime(run()).await
}
