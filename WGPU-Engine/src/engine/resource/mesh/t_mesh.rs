use wgpu::Buffer;

use crate::engine::{StandardInstance, TMaterial};

pub trait TMesh {
    fn vertex_buffer(&self) -> &Buffer;
    fn index_buffer(&self) -> &Buffer;
    fn index_count(&self) -> u32;
    fn instances(&mut self) -> &mut Vec<StandardInstance>;
    fn instance_count(&self) -> u32;
    fn instance_buffer(&self) -> &Buffer;
    fn material(&self) -> &dyn TMaterial;
    fn name(&self) -> Option<String>;
}
