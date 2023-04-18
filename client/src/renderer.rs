use serde::__private::size_hint::from_bounds;
use wgpu::{CommandEncoder, LoadOp, Operations, TextureUsages, TextureView};

use crate::{
    graphics::{Gpu, Texture},
    Game,
};

pub struct Renderer {
    framebuffer: TextureView,
    depth: TextureView,
}

impl Renderer {
    pub fn new(gpu: &Gpu) -> Self {
        let size = gpu.size();
        let framebuffer = Texture::new_uninit(
            gpu,
            size.width,
            size.height,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
        );
        let framebuffer = framebuffer.create_view(&Default::default());

        let depth = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Renderer.shadow_texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth = depth.create_view(&wgpu::TextureViewDescriptor {
            aspect: wgpu::TextureAspect::All,
            ..Default::default()
        });

        Self { framebuffer, depth }
    }

    pub fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView, game: &mut Game) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.0,
                        b: 0.2,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            // depth_stencil_attachment: None,
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(0.0),
                    store: true,
                }),
                // stencil_ops: None,
                stencil_ops: Some(Operations {
                    load: LoadOp::Load,
                    store: false,
                }),
            }),
        });

        game.render(&mut render_pass)
    }
}
