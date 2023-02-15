// mod game;
// pub mod graphics;
// pub mod renderer;
// pub mod utils;
// mod wasm;

#[no_mangle]
pub extern "C" fn run() -> i32 {
    println!("Hello from Rust");
    1
}

// #[no_mangle]
// pub extern "C" fn run_async() -> Box<dyn Future<Output = ()>> {
//     Box::new(async move {})
// }

// use std::{future::Future, time::Duration};

// pub use game::*;

// use wasm::{run, with_tokio_runtime};
// use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
