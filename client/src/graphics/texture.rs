use image::DynamicImage;
use wgpu::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

use super::Gpu;

pub struct Texture {
    texture: wgpu::Texture,
}

impl Texture {
    pub fn from_image(gpu: &Gpu, image: DynamicImage) -> Self {
        Self::new(
            gpu,
            image.width(),
            image.height(),
            TextureFormat::Rgba8UnormSrgb,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            &*image.to_rgba8(),
        )
    }

    pub fn new(
        gpu: &Gpu,
        width: u32,
        height: u32,
        format: TextureFormat,
        usage: TextureUsages,
        bytes: &[u8],
    ) -> Self {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = gpu.device.create_texture(&TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        gpu.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            bytes,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * width),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
            size,
        );

        Self { texture }
    }

    pub fn create_view(&self) -> TextureView {
        self.texture.create_view(&TextureViewDescriptor::default())
    }
}
