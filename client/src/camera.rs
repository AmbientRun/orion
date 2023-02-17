use glam::Mat4;

#[repr(C)]
#[derive(bytemuck::Zeroable, bytemuck::Pod, Copy, Debug, Clone)]
pub struct Camera {
    view: Mat4,
    proj: Mat4,
}

impl Camera {
    pub fn new(view: Mat4, proj: Mat4) -> Self {
        Self { view, proj }
    }
}
