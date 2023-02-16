use std::{f32::consts::TAU, sync::Arc};

use glam::{vec2, Vec2};
use itertools::Itertools;
use orion_shared::Asteroid;
use rand::{thread_rng, Rng};
use wgpu::{BindGroup, RenderPass, Sampler, TextureView};

use crate::graphics::{Gpu, Mesh, Shader, ShaderDesc, Texture, Vertex};

pub struct Game {
    asteroids: Vec<Asteroid>,
    gpu: Arc<Gpu>,
    shader: Shader,
    square: Mesh,
    asteroid_texture: TextureView,
    asteroid_bind_group: BindGroup,
    sampler: Sampler,
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

        let object_layout = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let square = Mesh::square(&gpu);
        let image = image::load_from_memory(include_bytes!("../assets/asteroid.png")).unwrap();
        let asteroid_texture = Texture::from_image(&gpu, image).create_view();

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let asteroid_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &object_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&asteroid_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let shader = Shader::new(
            &gpu,
            ShaderDesc {
                source: include_str!("../assets/shaders.wgsl").into(),
                format: gpu.surface_format(),
                vertex_layouts: vec![Vertex::layout()].into(),
            },
        );

        Self {
            asteroids,
            gpu,
            shader,
            square,
            asteroid_texture,
            sampler,
            asteroid_bind_group,
        }
    }

    pub fn update(&mut self, dt: f32) {
        for v in &mut self.asteroids {
            v.pos += v.vel * dt;
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(self.shader.pipeline());
        self.square.bind(render_pass);

        render_pass.set_bind_group(0, &self.asteroid_bind_group, &[]);

        render_pass.draw_indexed(0..self.square.index_count(), 0, 0..1);
    }
}
