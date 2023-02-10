use std::{f32::consts::TAU, f64::consts::PI};

use glam::{vec2, Vec2, Vec3};
use itertools::Itertools;
use orion_shared::Asteroid;
use rand::{thread_rng, Rng};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast};
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub struct Game {
    asteroids: Vec<Asteroid>,
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Self {
        let mut rng = thread_rng();

        let asteroids = (0..16)
            .map(|_| {
                let dir = rng.gen_range(0.0..TAU);
                let vel = vec2(dir.cos(), dir.sin()) * rng.gen_range(0.0..2.0);
                Asteroid {
                    size: rng.gen_range(10.0..20.0),
                    color: rng.gen(),
                    pos: rng.gen::<Vec2>() * 512.0,
                    vel,
                }
            })
            .collect_vec();

        Self { asteroids }
    }

    pub fn update(&mut self, dt: f32) {
        for v in &mut self.asteroids {
            v.pos += v.vel * dt;
        }
    }

    pub fn render(&self, canvas: HtmlCanvasElement) {
        eprintln!("Rendering");

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        for v in &self.asteroids {
            context.begin_path();

            let color = format!(
                "rgb({},{},{})",
                v.color.x * 255.0,
                v.color.y * 255.0,
                v.color.z * 255.0
            );
            let color = wasm_bindgen::JsValue::from_str(&color);

            context.set_fill_style(&color);
            context
                .arc(
                    v.pos.x as _,
                    v.pos.y as _,
                    v.size as _,
                    0.0,
                    std::f64::consts::TAU,
                )
                .unwrap();

            context.fill()
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
