use wgpu::{Adapter, Device, Instance, Queue};

use crate::engine::LogicalDevice;

pub trait TComputingEngine {
    fn get_instance(&self) -> &Instance;
    fn get_adapter(&self) -> &Adapter;
    fn get_logical_device(&self) -> &LogicalDevice;

    fn get_device(&self) -> &Device {
        self.get_logical_device().device()
    }

    fn get_queue(&self) -> &Queue {
        self.get_logical_device().queue()
    }
}
