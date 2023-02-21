use std::{sync::Arc, time::Duration};

use anyhow::Context;
use futures::FutureExt;
use tokio::select;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    fmt::time::UtcTime, prelude::__tracing_subscriber_SubscriberExt, registry,
    util::SubscriberInitExt, EnvFilter,
};
use tracing_web::*;
use utils::{
    task::spawn,
    timer::{self, sleep},
};
use wasm_bindgen::prelude::*;
use web_sys::window;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    platform::web::EventLoopExtWebSys,
    window::{Window, WindowBuilder},
};

use crate::{game::Game, graphics::Gpu, renderer::Renderer};

pub mod assets;
mod camera;
mod game;
pub mod graphics;
pub mod renderer;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub async fn start() {
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

    utils::set_panic_hook();

    spawn(timer::TimerWheel::new().start());

    tracing::info!("Sleeping...");
    sleep(Duration::from_secs(1)).await;
    tracing::info!("Finished sleeping");

    return;

    match run().await {
        Ok(()) => {}
        Err(err) => tracing::error!("{err:?}"),
    }
}

pub async fn run() -> anyhow::Result<()> {
    tracing::info!("Running app");

    //     let mut a = utils::task::spawn(async {
    //         sleep(Duration::from_millis(500)).await;
    //     });

    //     let mut b = utils::task::spawn(async {
    //         sleep(Duration::from_millis(1000)).await;
    //     });

    //     spawn(async move {
    //         loop {
    //             tokio::select! {
    //                 _ = &mut a => {
    //                     tracing::info!("A completed");
    //                 },
    //                 _ = &mut b => {
    //                     tracing::info!("B completed");
    //                 }
    //             }
    //         }
    //     });

    let event_loop = EventLoop::new();

    let perf = window()
        .context("Missing window")?
        .performance()
        .context("Performance missing")?;

    let window = WindowBuilder::new()
        .with_title("Winit window")
        .build(&event_loop)
        .unwrap();

    insert_canvas(&window);

    let gpu = Arc::new(Gpu::new(window).await);
    let mut renderer = Renderer::new();

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

            // RedrawRequested will only trigger once, unless we manually
            // request it.
            gpu.window().request_redraw();
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
    canvas.style().set_css_text("background-color: crimson;");
    body.append_child(&canvas).unwrap();
}
