use wgpu::{Adapter, Device, Instance, Queue};

use crate::engine::LogicalDevice;

pub trait TComputingEngine {
    fn instance(&self) -> &Instance;
    fn adapter(&self) -> &Adapter;
    fn logical_device(&self) -> &LogicalDevice;

    fn device(&self) -> &Device {
        self.logical_device().device()
    }

    fn queue(&self) -> &Queue {
        self.logical_device().queue()
    }
}
