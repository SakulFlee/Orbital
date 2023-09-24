use bytemuck::NoUninit;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

pub trait BufferHelper {
    fn make_buffer<A>(&self, label: Option<&str>, content: &[A], usage: BufferUsages) -> Buffer
    where
        A: NoUninit;

    fn make_buffer_<A>(
        label: Option<&str>,
        content: &[A],
        usage: BufferUsages,
        device: &Device,
    ) -> Buffer
    where
        A: NoUninit,
    {
        device.create_buffer_init(&BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(&content),
            usage,
        })
    }
}

impl BufferHelper for Device {
    fn make_buffer<A>(&self, label: Option<&str>, content: &[A], usage: BufferUsages) -> Buffer
    where
        A: NoUninit,
    {
        Self::make_buffer_(label, content, usage, &self)
    }
}
