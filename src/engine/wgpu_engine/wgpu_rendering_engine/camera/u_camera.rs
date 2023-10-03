use bytemuck::{Pod, Zeroable};

use super::Camera;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct UCamera {
    position: [f32; 4],
    view_projection_matrix: [[f32; 4]; 4],
}

impl UCamera {
    pub fn new(position: [f32; 4], view_projection_matrix: [[f32; 4]; 4]) -> Self {
        Self {
            position,
            view_projection_matrix,
        }
    }

    #[rustfmt::skip]
    pub fn empty() -> Self {
        Self::new([0.0, 0.0, 0.0, 0.0],[
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
        ])
    }

    pub fn from_camera(camera: &Camera) -> Self {
        Self {
            position: [camera.position.x, camera.position.y, camera.position.z, 0.0],
            view_projection_matrix: camera.calculate_matrix().into(),
        }
    }
}
