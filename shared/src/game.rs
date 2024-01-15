use std::{f32::consts::TAU, sync::Arc};

use bytemuck::{Pod, Zeroable};

use glam::{vec2, vec3, Mat4, Quat, Vec2, Vec3, Vec4};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use wgpu::{BindGroup, BufferUsages, IndexFormat, RenderPass, ShaderStages, TextureView};

use crate::{
    camera::Camera,
    graphics::{
        BindGroupBuilder, BindGroupLayoutBuilder, Gpu, Mesh, Shader, ShaderDesc, Texture,
        TypedBuffer, Vertex,
    },
};

pub struct Asteroid {
    pub color: Vec3,
    pub radius: f32,
    pub pos: Vec2,
    pub rot: f32,
    pub ang_vel: f32,
    pub lifetime: f32,
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Default, Debug)]
struct Object {
    model: Mat4,
    color: Vec4,
}

#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct DrawIndexedIndirect {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: u32,
    pub first_instance: u32,
}

pub struct Game {
    asteroids: Vec<Asteroid>,
    object_data: Vec<Object>,
    gpu: Arc<Gpu>,
    shader: Shader,
    square: Mesh,
    camera_buffer: TypedBuffer<Camera>,
    asteroid_texture: TextureView,
    indirect_buffer: TypedBuffer<DrawIndexedIndirect>,

    object_buffers: Vec<TypedBuffer<Object>>,
    asteroid_bind_groups: Vec<BindGroup>,

    sampler: wgpu::Sampler,

    orbit_time: f32,
    asteroid_bind_group_layout: wgpu::BindGroupLayout,
}

impl Game {
    pub async fn new(gpu: Arc<Gpu>) -> anyhow::Result<Self> {
        let square = Mesh::square(&gpu);

        tracing::info!("Downloading asteroid texture");

        let image = reqwest::Client::builder()
            .build()?
            .get("https://dims-content.fra1.digitaloceanspaces.com/assets%2Forion%2Fasteroid.png")
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

        let indirect_buffer = TypedBuffer::new(
            &gpu,
            "indirect_buffer",
            BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            &[DrawIndexedIndirect::zeroed(); 16],
        );

        let asteroid_bind_group_layout = BindGroupLayoutBuilder::new("asteroid_bind_group_layout")
            .bind_uniform_buffer(ShaderStages::VERTEX)
            .bind_uniform_buffer(ShaderStages::VERTEX)
            .bind_texture(ShaderStages::FRAGMENT)
            .bind_sampler(ShaderStages::FRAGMENT)
            .build(&gpu);

        let shader = Shader::new(
            &gpu,
            ShaderDesc {
                label: "asteroids",
                source: include_str!("../../assets/shaders.wgsl").into(),
                format: gpu.surface_format(),
                vertex_layouts: vec![Vertex::layout()].into(),
                layouts: &[&asteroid_bind_group_layout],
            },
        );

        Ok(Self {
            asteroids: Vec::new(),
            gpu,
            shader,
            square,
            asteroid_bind_groups: Vec::new(),
            asteroid_bind_group_layout,
            object_buffers: Vec::new(),
            object_data,
            indirect_buffer,
            orbit_time: 0.0,
            camera_buffer,
            asteroid_texture,
            sampler,
        })
    }

    pub fn update(&mut self, dt: f32) {
        self.orbit_time += dt;
        let layers = [2, 8, 32, 48, 64, 96];

        let mut rng = Pcg32::from_entropy();
        let total_count: usize = layers.iter().sum();

        let orbit_time = self.orbit_time;

        self.asteroids
            .extend((self.asteroids.len()..total_count).map(|_| Asteroid {
                radius: 0.2,
                color: rng.gen(),
                pos: Vec2::ZERO,
                rot: rng.gen_range(0.0..=TAU),
                ang_vel: rng.gen_range(-1.0..=1.0),
                lifetime: rng.gen_range(1.0..=10.0),
            }));

        tracing::info!(asteroid_count = self.asteroids.len());

        layers
            .iter()
            .enumerate()
            .flat_map(|(i, &count)| {
                (0..count).map(move |v| (i as f32 + 1.0, v as f32 / count as f32 * TAU))
            })
            .zip(&mut self.asteroids)
            .for_each(|((layer, theta), asteroid)| {
                let theta = theta + orbit_time * layer.powi(3).sqrt().recip() * 1.0;

                let ang = vec2(theta.cos(), theta.sin()) * layer.powi(2) * 0.5;
                asteroid.pos = ang;

                asteroid.rot += asteroid.ang_vel * dt;
            });

        // self.spawner.update(&mut self.asteroids, dt);
    }

    pub fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>) {
        if self.asteroids.is_empty() {
            return;
        }

        let mut cmds = [DrawIndexedIndirect::zeroed(); 512];
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

                // let cmd = DrawIndexedIndirect {
                //     index_count,
                //     instance_count: 1,
                //     base_vertex: 0,
                //     first_index: 0,
                //     first_instance: i as u32,
                // };

                // cmds[i] = cmd;
            });

        const CHUNK_SIZE: usize = 128;
        for i in self.object_buffers.len()..(self.asteroids.len().div_ceil(CHUNK_SIZE)) {
            tracing::info!("creating buffer {i}");
            // Uniform buffers have a strict byte size limit
            let object_data = vec![Default::default(); CHUNK_SIZE];

            let object_buffer = TypedBuffer::new(
                &self.gpu,
                "object_buffer",
                BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                &object_data,
            );

            let asteroid_bind_group = BindGroupBuilder::new("asteroid_bind_group")
                .bind_buffer(&self.camera_buffer)
                .bind_buffer(&object_buffer)
                .bind_texture(&self.asteroid_texture)
                .bind_sampler(&self.sampler)
                .build(&self.gpu, &self.asteroid_bind_group_layout);

            self.object_buffers.push(object_buffer);
            self.asteroid_bind_groups.push(asteroid_bind_group);
        }

        // The uniform buffer is limited in size, so we need to split the data into chunks and
        // separately write them to the buffer and issue draw calls
        for (i, chunk) in self.asteroids.chunks(CHUNK_SIZE).enumerate() {
            // The buffer is enqueued for writing to the GPU
            //
            // As such, we need a ring of them
            let chunk_start = i * CHUNK_SIZE;
            self.object_buffers[i].write(
                &self.gpu.queue,
                &self.object_data[chunk_start..chunk_start + chunk.len()],
            );
            // self.indirect_buffer.write(&self.gpu.queue, &cmds.);

            render_pass.set_pipeline(self.shader.pipeline());

            render_pass.set_bind_group(0, &self.asteroid_bind_groups[i], &[]);

            render_pass.set_vertex_buffer(0, self.square.vertex_buffer.slice(..));

            render_pass.set_index_buffer(self.square.index_buffer.slice(..), IndexFormat::Uint32);

            // for i in 0..cmds.len() {
            // render_pass.draw_indexed_indirect(
            //     &self.indirect_buffer,
            //     i as u64 * std::mem::size_of::<DrawIndexedIndirect>() as u64,
            // );
            // }

            render_pass.draw_indexed(
                0..index_count,
                0,
                chunk_start as u32..(chunk_start + chunk.len()) as u32,
            );
        }
    }
}
