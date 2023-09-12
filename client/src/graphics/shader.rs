use std::borrow::Cow;

use wgpu::{
    BindGroupLayout, DepthStencilState, PipelineLayoutDescriptor, RenderPipeline, StencilState,
    TextureFormat, VertexBufferLayout,
};

use super::Gpu;
#[derive(Debug, Clone)]
pub struct ShaderDesc<'a> {
    pub label: &'a str,
    pub source: Cow<'a, str>,
    pub format: TextureFormat,
    pub vertex_layouts: Cow<'a, [VertexBufferLayout<'static>]>,
    pub layouts: &'a [&'a BindGroupLayout],
}

pub struct Shader {
    pipeline: RenderPipeline,
}

impl Shader {
    pub fn new(gpu: &Gpu, desc: ShaderDesc) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(desc.source),
            });

        let layout = gpu
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some(desc.label),
                bind_group_layouts: desc.layouts,
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(desc.label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",        // 1.
                    buffers: &desc.vertex_layouts, // 2.
                },
                fragment: Some(wgpu::FragmentState {
                    // 3.
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: desc.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw, // 2.
                    // cull_mode: Some(wgpu::Face::Back),
                    cull_mode: None,
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            });

        Self { pipeline }
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }
}
