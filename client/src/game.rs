use std::{f32::consts::TAU, sync::Arc};

use bytemuck::{Pod, Zeroable};
use futures::task::SpawnExt;
use glam::{vec2, vec3, vec4, Mat4, Quat, Vec2, Vec3, Vec4};
use itertools::Itertools;
use orion_shared::Asteroid;
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};
use rand_pcg::Pcg32;
use wgpu::{BindGroup, BufferUsages, RenderPass, Sampler, ShaderStages, TextureView};

use crate::{
    camera::Camera,
    graphics::{
        BindGroupBuilder, BindGroupLayoutBuilder, Gpu, Mesh, Shader, ShaderDesc, Texture,
        TypedBuffer, Vertex,
    },
};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Default, Debug)]
struct Object {
    model: Mat4,
    color: Vec4,
}

pub struct Spawner {
    acc: f32,
    spawn_interval: f32,
    spawn_count: usize,
    rng: rand_pcg::Pcg32,
}

impl Spawner {
    fn update(&mut self, asteroids: &mut Vec<Asteroid>, dt: f32) {
        asteroids.retain_mut(|v| {
            v.lifetime -= dt;
            v.lifetime >= 0.0
        });

        self.acc += dt;
        while self.acc >= self.spawn_interval {
            for _ in 0..self.spawn_count {
                self.spawn(asteroids);
            }

            self.acc -= self.spawn_interval;
        }
    }

    fn spawn(&mut self, asteroids: &mut Vec<Asteroid>) {
        let rng = &mut self.rng;

        let dir = rng.gen_range(0.0..TAU);
        let vel = vec2(dir.cos(), dir.sin()) * rng.gen_range(0.0..5.0);

        asteroids.push(Asteroid {
            radius: rng.gen_range(0.2..=1.0),
            color: rng.gen(),
            pos: Vec2::ZERO,
            vel,
            rot: rng.gen_range(0.0..=TAU),
            ang_vel: rng.gen_range(-1.0..=1.0),
            lifetime: rng.gen_range(1.0..=10.0),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct DrawIndexedIndirect {
    /// The number of vertices to draw.
    pub vertex_count: u32,
    /// The number of instances to draw.
    pub instance_count: u32,
    /// The base index within the index buffer.
    pub base_index: u32,
    /// The value added to the vertex index before indexing into the vertex buffer.
    pub vertex_offset: i32,
    /// The instance ID of the first instance to draw.
    /// Has to be 0, unless [`Features::INDIRECT_FIRST_INSTANCE`](crate::Features::INDIRECT_FIRST_INSTANCE) is enabled.
    pub base_instance: u32,
}

pub struct Game {
    asteroids: Vec<Asteroid>,
    object_data: Vec<Object>,
    gpu: Arc<Gpu>,
    shader: Shader,
    square: Mesh,
    asteroid_texture: TextureView,
    camera_buffer: TypedBuffer<Camera>,
    object_buffer: TypedBuffer<Object>,
    indirect_buffer: TypedBuffer<DrawIndexedIndirect>,

    asteroid_bind_group: BindGroup,
    sampler: Sampler,
    bounds: Vec2,

    spawner: Spawner,
}

impl Game {
    pub async fn new(gpu: Arc<Gpu>) -> anyhow::Result<Self> {
        let mut spawner = Spawner {
            acc: 0.0,
            spawn_interval: 1.0,
            spawn_count: 64,
            rng: Pcg32::from_entropy(),
        };

        let square = Mesh::square(&gpu);

        tracing::info!("Downloading asteroid texture");

        let image = reqwest::Client::builder()
            .build()?
            .get("https://dims-content.fra1.digitaloceanspaces.com/assets%2Forion%2Fasteroid.png")
            // .get("https://upload.wikimedia.org/wikipedia/commons/thumb/e/ec/Mona_Lisa%2C_by_Leonardo_da_Vinci%2C_from_C2RMF_retouched.jpg/1200px-Mona_Lisa%2C_by_Leonardo_da_Vinci%2C_from_C2RMF_retouched.jpg")
            .send()
            .await?
            .bytes()
            .await?;

        tracing::info!("Downloaded asteroid texture");

        let image = image::load_from_memory(&image).unwrap();

        let asteroid_texture = Texture::from_image(&gpu, image).create_view(&Default::default());

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            ..Default::default()
        });

        let size = 10.0;
        let window_size = gpu.window().inner_size();

        let aspect = window_size.width as f32 / window_size.height as f32;

        let bounds = vec2(size * aspect, size);

        tracing::info!(?bounds, "bounds");
        let camera = Camera::new(
            Mat4::from_rotation_translation(Quat::IDENTITY, vec3(0.0, 0.0, 1.0)),
            Mat4::orthographic_lh(-bounds.x, bounds.x, -bounds.y, bounds.y, 0.1, 1000.0),
        );

        let camera_buffer = TypedBuffer::new(
            &gpu,
            "camera_buffer",
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            &[camera],
        );

        let object_data = vec![Default::default(); 1024];

        let object_buffer = TypedBuffer::new(
            &gpu,
            "object_buffer",
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            &object_data,
        );

        let indirect_buffer = TypedBuffer::new(
            &gpu,
            "indirect_buffer",
            BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            &[DrawIndexedIndirect::zeroed(); 1024],
        );

        let asteroid_bind_group_layout = BindGroupLayoutBuilder::new("asteroid_bind_group_layout")
            .bind_uniform_buffer(ShaderStages::VERTEX)
            .bind_storage_buffer(ShaderStages::VERTEX)
            .bind_texture(ShaderStages::FRAGMENT)
            .bind_sampler(ShaderStages::FRAGMENT)
            .build(&gpu);

        let asteroid_bind_group = BindGroupBuilder::new("asteroid_bind_group")
            .bind_buffer(&camera_buffer)
            .bind_buffer(&object_buffer)
            .bind_texture(&asteroid_texture)
            .bind_sampler(&sampler)
            .build(&gpu, &asteroid_bind_group_layout);

        let shader = Shader::new(
            &gpu,
            ShaderDesc {
                label: "asteroids",
                source: include_str!("../assets/shaders.wgsl").into(),
                format: gpu.surface_format(),
                vertex_layouts: vec![Vertex::layout()].into(),
                layouts: &[&asteroid_bind_group_layout],
            },
        );

        let mut asteroids = Vec::new();
        for _ in 0..16 {
            spawner.spawn(&mut asteroids)
        }

        Ok(Self {
            asteroids,
            gpu,
            shader,
            square,
            asteroid_texture,
            sampler,
            asteroid_bind_group,
            camera_buffer,
            object_data,
            object_buffer,
            bounds,
            spawner,
            indirect_buffer,
        })
    }

    pub fn update(&mut self, dt: f32) {
        for v in &mut self.asteroids {
            v.pos += v.vel * dt;
            v.rot += v.ang_vel * dt;

            // Handle wall collision

            // let right = self.bounds.x;
            // let top = self.bounds.y;

            // if v.pos.x + v.radius > right {
            //     v.vel = vec2(-v.vel.x.abs(), v.vel.y);
            // }
            // if v.pos.x - v.radius < -right {
            //     v.vel = vec2(v.vel.x.abs(), v.vel.y);
            // }

            // if v.pos.y + v.radius > top {
            //     v.vel = vec2(v.vel.x, -v.vel.y.abs());
            // }
            // if v.pos.y - v.radius < -top {
            //     v.vel = vec2(v.vel.x, v.vel.y.abs());
            // }
        }

        // self.spawner.update(&mut self.asteroids, dt);
    }

    pub fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>) {
        if self.asteroids.is_empty() {
            return;
        }

        let mut cmds = Vec::new();
        let index_count = self.square.index_count();
        // Update the object data
        self.object_data
            .iter_mut()
            .zip(&self.asteroids)
            .enumerate()
            .for_each(|(i, (object, v))| {
                object.model = Mat4::from_scale_rotation_translation(
                    Vec3::ONE * 2.0 * v.radius,
                    Quat::from_scaled_axis(Vec3::Z * v.rot),
                    v.pos.extend(0.0),
                );
                object.color = Vec4::ONE * v.lifetime.clamp(0.0, 1.0);
                cmds.push(DrawIndexedIndirect {
                    vertex_count: index_count,
                    instance_count: 1,
                    base_index: 0,
                    vertex_offset: 0,
                    base_instance: i as _,
                })
            });

        self.object_buffer.write(&self.gpu.queue, &self.object_data);
        self.indirect_buffer.write(&self.gpu.queue, &cmds);

        render_pass.set_pipeline(self.shader.pipeline());

        render_pass.set_bind_group(0, &self.asteroid_bind_group, &[]);

        self.square.bind(render_pass);

        for i in 0..cmds.len() {
            render_pass.draw_indexed_indirect(
                &self.indirect_buffer,
                i as u64 * std::mem::size_of::<DrawIndexedIndirect>() as u64,
            );
        }

        // render_pass.draw_indexed(
        //     0..self.square.index_count(),
        //     0,
        //     0..self.asteroids.len() as _,
        // );
    }
}
