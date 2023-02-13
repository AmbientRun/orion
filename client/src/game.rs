use std::{f32::consts::TAU, f64::consts::PI, sync::Arc};

use glam::{vec2, Vec2, Vec3};
use itertools::Itertools;
use orion_shared::Asteroid;
use rand::{thread_rng, Rng};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast};
use web_sys::HtmlCanvasElement;
use wgpu::{CommandEncoder, RenderPass};

use crate::graphics::{Gpu, Mesh, Shader, ShaderDesc, Vertex};

pub struct Game {
    asteroids: Vec<Asteroid>,
    gpu: Arc<Gpu>,
    shader: Shader,
    square: Mesh,
}

impl Game {
    pub fn new(gpu: Arc<Gpu>) -> Self {
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

        let shader = Shader::new(
            &gpu,
            ShaderDesc {
                source: include_str!("../assets/shaders.wgsl").into(),
                format: gpu.surface_format(),
                vertex_layouts: vec![Vertex::layout()].into(),
            },
        );

        let square = Mesh::square(&gpu);

        Self {
            asteroids,
            gpu,
            shader,
            square,
        }
    }

    pub fn update(&mut self, dt: f32) {
        for v in &mut self.asteroids {
            v.pos += v.vel * dt;
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        tracing::info!("Drawing game");
        render_pass.set_pipeline(self.shader.pipeline());
        self.square.bind(render_pass);
        render_pass.draw_indexed(0..self.square.index_count(), 0, 0..1);
    }
}
