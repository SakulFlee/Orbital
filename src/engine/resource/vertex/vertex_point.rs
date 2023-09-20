use bytemuck::{Pod, Zeroable};

use super::TVertex;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct VertexPoint {
    pub position_coordinates: [f32; 3],
    pub texture_coordinates: [f32; 2],
    pub normal_coordinates: [f32; 3],
}

impl TVertex for VertexPoint {
    fn get_position_coordinates(&self) -> [f32; 3] {
        self.position_coordinates
    }

    fn get_texture_coordinates(&self) -> [f32; 2] {
        self.texture_coordinates
    }

    fn get_normal_coordinates(&self) -> [f32; 3] {
        self.normal_coordinates
    }
}
