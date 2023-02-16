use std::{sync::Arc, time::Duration};

use time::*;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    fmt::time::UtcTime, prelude::__tracing_subscriber_SubscriberExt, registry,
    util::SubscriberInitExt, EnvFilter,
};
use tracing_web::*;
use wasm_bindgen::prelude::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    platform::web::EventLoopExtWebSys,
    window::{Window, WindowBuilder},
};

use crate::{game::Game, graphics::Gpu, renderer::Renderer};

mod game;
pub mod graphics;
pub mod renderer;
mod util;
// mod wasm;

// pub extern "C" fn run_async() -> Box<dyn Future<Output = ()>> {
//     Box::new(async move {})
// }

// use std::{future::Future, time::Duration};

// pub use game::*;

// use tracing::metadata::LevelFilter;
// use tracing_subscriber::{
//     fmt::time::UtcTime, prelude::__tracing_subscriber_SubscriberExt, registry,
//     util::SubscriberInitExt, EnvFilter,
// };
// use tracing_web::MakeConsoleWriter;
// use wasm::with_tokio_runtime;
// use wasm_bindgen::prelude::wasm_bindgen;
// use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub async fn run() {
    util::set_panic_hook();
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter); // write events to the console

    registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(fmt_layer)
        .init();

    tracing::info!("Running app");

    let task1 = util::spawn(async move {
        loop {
            gloo::timers::future::sleep(Duration::from_millis(1000)).await;
            tracing::info!("Hello again");
            // async_std::task::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    let task2 = util::spawn(async move {
        gloo::timers::future::sleep(Duration::from_millis(5000)).await;
        "Hello from setTimeout".to_string()
    });

    let task3 = util::spawn(futures::future::pending::<()>());

    let result = task2.await;
    tracing::info!("Got: {result:?}");
    task3.abort();

    tracing::info!("Got task3: {:?}", task3.await);

    tracing::info!("Finished waiting on tasks");

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Winit window")
        .build(&event_loop)
        .unwrap();

    insert_canvas(&window);

    let gpu = Arc::new(Gpu::new(window).await);
    let mut renderer = Renderer::new();

    let game = Game::new(gpu.clone());

    event_loop.spawn(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == gpu.window().id() => match event {
            WindowEvent::ReceivedCharacter(c) => {
                tracing::info!("Typed: {c}");
            }
            WindowEvent::Resized(physical_size) => {
                gpu.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                // new_inner_size is &&mut so we have to dereference it twice
                gpu.resize(**new_inner_size);
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == gpu.window().id() => {
            match gpu.render(|encoder, view| renderer.render(encoder, view, &game)) {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => gpu.resize(gpu.size()),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            gpu.window().request_redraw();
        }
        _ => {}
    });
}

pub fn insert_canvas(window: &Window) {
    use winit::platform::web::WindowExtWebSys;

    let canvas = window.canvas();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    // Set a background color for the canvas to make it easier to tell where the canvas is for debugging purposes.
    canvas.style().set_css_text("background-color: crimson;");
    body.append_child(&canvas).unwrap();
}
