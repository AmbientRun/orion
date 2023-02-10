use tracing_subscriber::{prelude::*, registry};
use tracing_tree::HierarchicalLayer;
use tracing_web::MakeConsoleWriter;
use wasm_bindgen::prelude::wasm_bindgen;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

#[wasm_bindgen(start)]
pub fn run() {
    registry()
        .with(HierarchicalLayer::new(4).with_writer(MakeConsoleWriter))
        .init();

    tracing::info!("Initializing app");

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Winit window")
        .build(&event_loop)
        .unwrap();

    insert_canvas(&window);
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
