use wgpu::{CommandEncoder, TextureView};

use crate::Game;

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView, game: &Game) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
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

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
