use serde::__private::size_hint::from_bounds;
use wgpu::{CommandEncoder, TextureUsages, TextureView};

use crate::{
    graphics::{Gpu, Texture},
    Game,
};

pub struct Renderer {
    framebuffer: TextureView,
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
        Self { framebuffer }
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
            depth_stencil_attachment: None,
        });

        game.render(&mut render_pass)
    }
}
