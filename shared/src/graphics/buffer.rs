use std::marker::PhantomData;

use bytemuck::Pod;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Queue,
};

use super::Gpu;

pub struct TypedBuffer<T> {
    buffer: Buffer,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T> std::ops::Deref for TypedBuffer<T> {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<T> TypedBuffer<T>
where
    T: Pod,
{
    pub fn new(gpu: &Gpu, label: &str, usage: BufferUsages, data: &[T]) -> Self {
        let buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage,
        });

        Self {
            buffer,
            len: data.len(),
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn write(&self, queue: &Queue, data: &[T]) {
        assert!(data.len() <= self.len);
        queue.write_buffer(self, 0, bytemuck::cast_slice(data));
    }
}
