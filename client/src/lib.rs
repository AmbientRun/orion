use std::sync::Arc;

use anyhow::Context;

use shared::{game::Game, graphics::Gpu, renderer::Renderer};
use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    fmt::time::UtcTime, prelude::__tracing_subscriber_SubscriberExt, registry,
    util::SubscriberInitExt, EnvFilter,
};
use tracing_web::*;

use wasm_bindgen::prelude::*;
use web_sys::window;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    platform::web::EventLoopExtWebSys,
    window::{Window, WindowBuilder},
};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub async fn start() {
    console_error_panic_hook::set_once();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter); // write events to the console

    registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .with(fmt_layer)
        .init();

    match run().await {
        Ok(()) => {}
        Err(err) => tracing::error!("{err:?}"),
    }
}

pub async fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();

    let perf = window()
        .context("Missing window")?
        .performance()
        .context("Performance missing")?;

    // let canvas = window()
    //     .unwrap()
    //     .document()
    //     .unwrap()
    //     .get_element_by_id("canvas")
    //     .unwrap();

    let window = WindowBuilder::new()
        .with_title("Winit window")
        // .with_canvas(Some(canvas.dyn_into().unwrap()))
        .build(&event_loop)
        .unwrap();

    insert_canvas(&window);

    let gpu = Arc::new(Gpu::new(window).await);
    let mut renderer = Renderer::new(&gpu);

    let mut game = Game::new(gpu.clone()).await.unwrap();

    let mut current_time = perf.now() / 1000.0;
    let mut acc = 0.0;
    let dt = 1.0 / 50.0;

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
            match gpu.render(|encoder, view| renderer.render(encoder, view, &mut game)) {
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
            let new_time = perf.now() / 1000.0;

            let frame_time = new_time - current_time;
            current_time = new_time;

            acc += frame_time;

            while acc >= dt {
                game.update(dt as _);
                acc -= dt;
            }

            match gpu.render(|encoder, view| renderer.render(encoder, view, &mut game)) {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => gpu.resize(gpu.size()),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        _ => {}
    });

    Ok(())
}

pub fn insert_canvas(window: &Window) {
    use winit::platform::web::WindowExtWebSys;

    let canvas = window.canvas();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    // Set a background color for the canvas to make it easier to tell where the canvas is for debugging purposes.
    // canvas.style().set_css_text("background-color: crimson;");
    body.append_child(&canvas).unwrap();
}
