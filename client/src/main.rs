fn main() {
    eprintln!("Running main");
    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    // with_tokio_runtime(run()).await
}
