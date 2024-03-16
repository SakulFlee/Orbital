pub use logical_device::LogicalDevice;
use wgpu::{Device, Queue};

pub type LogicalDeviceWGPU = LogicalDevice<Device, Queue>;
