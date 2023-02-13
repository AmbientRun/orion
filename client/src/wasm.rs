use std::sync::Arc;

use tracing::{info_span, metadata::LevelFilter};
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

#[wasm_bindgen(start)]
pub async fn run() {
    utils::set_panic_hook();
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

    tracing::info!("Initializing app");

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