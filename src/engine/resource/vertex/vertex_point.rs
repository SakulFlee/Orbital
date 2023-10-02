use bytemuck::{Pod, Zeroable};

use super::TVertex;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct VertexPoint {
    pub position_coordinates: [f32; 3],
    pub texture_coordinates: [f32; 2],
    pub normal_coordinates: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl TVertex for VertexPoint {
    fn position_coordinates(&self) -> [f32; 3] {
        self.position_coordinates
    }

    fn texture_coordinates(&self) -> [f32; 2] {
        self.texture_coordinates
    }

    fn normal_coordinates(&self) -> [f32; 3] {
        self.normal_coordinates
    }

    fn tangent(&self) -> [f32; 3] {
        self.tangent
    }
    
    fn bitangent(&self) -> [f32; 3] {
        self.bitangent
    }
}
