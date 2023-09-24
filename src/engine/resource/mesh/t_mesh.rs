use std::ops::Range;

use wgpu::Buffer;

use crate::engine::TMaterial;

pub trait TMesh {
    fn get_vertex_buffer(&self) -> &Buffer;
    fn get_index_buffer(&self) -> &Buffer;
    fn get_index_count(&self) -> u32;
    fn get_instance_range(&self) -> Range<u32>;
    fn set_instance_range(&mut self, range: Range<u32>);
    fn get_material(&self) -> Option<&Box<dyn TMaterial>>;
    fn get_name(&self) -> Option<String>;
}
