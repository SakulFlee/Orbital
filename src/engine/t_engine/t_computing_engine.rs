use wgpu::{Adapter, Device, Instance, Queue};

pub trait TComputingEngine {
    fn get_instance(&self) -> &Instance;
    fn get_adapter(&self) -> &Adapter;
    fn get_device(&self) -> &Device;
    fn get_queue(&self) -> &Queue;
}
