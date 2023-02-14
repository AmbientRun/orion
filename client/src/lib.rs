mod game;
pub mod graphics;
pub mod renderer;
mod utils;
mod wasm;

use std::time::Duration;

pub use game::*;

use wasm::{run, with_tokio_runtime};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
