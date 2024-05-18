use bytemuck::NoUninit;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

pub trait BufferUtil {
    fn make_buffer<T>(&self, label: Option<&str>, content: &[T], usage: BufferUsages) -> Buffer
    where
        T: NoUninit;
}

impl BufferUtil for Device {
    fn make_buffer<T>(&self, label: Option<&str>, content: &[T], usage: BufferUsages) -> Buffer
    where
        T: NoUninit,
    {
        self.create_buffer_init(&BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(content),
            usage,
        })
    }
}
