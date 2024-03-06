use wgpu::{Device, Queue};

pub struct LogicalDevice {
    device: Device,
    queue: Queue,
}

impl LogicalDevice {
    pub fn new(device: Device, queue: Queue) -> Self {
        Self { device, queue }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}
