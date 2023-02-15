use std::{future::Future, sync::Arc};

use once_cell::sync::Lazy;
use tokio::runtime::{self, Runtime};
use tracing::metadata::LevelFilter;
use tracing_subscriber::{fmt::time::UtcTime, prelude::*, registry, EnvFilter};
use tracing_web::MakeConsoleWriter;
use wasm_bindgen::prelude::wasm_bindgen;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    platform::web::EventLoopExtWebSys,
    window::{Window, WindowBuilder},
};

use crate::{graphics::Gpu, renderer::Renderer, utils, Game};

/// Provides a tokio runtime to the future without blocking the browser's executor
pub fn with_tokio_runtime(future: impl Future<Output = ()>) -> impl Future<Output = ()> {
    static RUNTIME: Lazy<Runtime> =
        Lazy::new(|| runtime::Builder::new_current_thread().build().unwrap());

    tokio_util::context::TokioContext::new(future, RUNTIME.handle().clone())
}

// pub async fn run() {
//     utils::set_panic_hook();
//     let fmt_layer = tracing_subscriber::fmt::layer()
//         .with_ansi(false) // Only partially supported across browsers
//         .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
//         .with_writer(MakeConsoleWriter); // write events to the console

//     registry()
//         .with(
//             EnvFilter::builder()
//                 .with_default_directive(LevelFilter::INFO.into())
//                 .from_env_lossy(),
//         )
//         .with(fmt_layer)
//         .init();

//     tracing::info!("Initializing app");

//     let task = tokio::spawn(async move {
//         tracing::info!("Hello from the other side");
//     });

//     tracing::info!("Spawned task");
//     task.await.unwrap();
//     tracing::info!("Task completed");

//     let event_loop = EventLoop::new();

//     let window = WindowBuilder::new()
//         .with_title("Winit window")
//         .build(&event_loop)
//         .unwrap();

//     insert_canvas(&window);

//     let gpu = Arc::new(Gpu::new(window).await);
//     let mut renderer = Renderer::new();

//     let game = Game::new(gpu.clone());

//     event_loop.spawn(move |event, _, control_flow| match event {
//         Event::WindowEvent {
//             ref event,
//             window_id,
//         } if window_id == gpu.window().id() => match event {
//             WindowEvent::ReceivedCharacter(c) => {
//                 tracing::info!("Typed: {c}");
//             }
//             WindowEvent::Resized(physical_size) => {
//                 gpu.resize(*physical_size);
//             }
//             WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
//                 // new_inner_size is &&mut so we have to dereference it twice
//                 gpu.resize(**new_inner_size);
//             }
//             WindowEvent::CloseRequested
//             | WindowEvent::KeyboardInput {
//                 input:
//                     KeyboardInput {
//                         state: ElementState::Pressed,
//                         virtual_keycode: Some(VirtualKeyCode::Escape),
//                         ..
//                     },
//                 ..
//             } => *control_flow = ControlFlow::Exit,
//             _ => {}
//         },
//         Event::RedrawRequested(window_id) if window_id == gpu.window().id() => {
//             match gpu.render(|encoder, view| renderer.render(encoder, view, &game)) {
//                 Ok(_) => {}
//                 // Reconfigure the surface if lost
//                 Err(wgpu::SurfaceError::Lost) => gpu.resize(gpu.size()),
//                 // The system is out of memory, we should probably quit
//                 Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
//                 // All other errors (Outdated, Timeout) should be resolved by the next frame
//                 Err(e) => eprintln!("{:?}", e),
//             }
//         }
//         Event::MainEventsCleared => {
//             // RedrawRequested will only trigger once, unless we manually
//             // request it.
//             gpu.window().request_redraw();
//         }
//         _ => {}
//     });
// }
