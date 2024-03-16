#[derive(Debug)]
pub struct LogicalDevice<Device, Queue> {
    device: Device,
    queue: Queue,
}

impl<Device, Queue> LogicalDevice<Device, Queue> {
    pub fn new(device: Device, queue: Queue) -> Self {
        Self { device, queue }
    }

    pub fn device<'a>(&'a self) -> &'a Device
    where
        Queue: 'a,
    {
        &self.device
    }

    pub fn queue<'a>(&'a self) -> &'a Queue
    where
        Device: 'a,
    {
        &self.queue
    }
}
