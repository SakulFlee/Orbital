use wgpu::Buffer;

use crate::engine::{StandardInstance, TMaterial};

pub trait TMesh {
    fn get_vertex_buffer(&self) -> &Buffer;
    fn get_index_buffer(&self) -> &Buffer;
    fn get_index_count(&self) -> u32;
    fn get_instances(&mut self) -> &mut Vec<StandardInstance>;
    fn get_instance_count(&self) -> u32;
    fn get_instance_buffer(&self) -> &Buffer;
    fn get_material(&self) -> &dyn TMaterial;
    fn get_name(&self) -> Option<String>;
}
