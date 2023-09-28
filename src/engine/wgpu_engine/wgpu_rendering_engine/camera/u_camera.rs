use bytemuck::{Pod, Zeroable};

use super::Camera;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct UCamera {
    view_projection_matrix: [[f32; 4]; 4],
}

impl UCamera {
    pub fn new(view_projection_matrix: [[f32; 4]; 4]) -> Self {
        Self {
            view_projection_matrix,
        }
    }

    #[rustfmt::skip]
    pub fn empty() -> Self {
        Self::new([
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
        ])
    }

    pub fn from_camera(camera: &Camera) -> Self {
        Self {
            view_projection_matrix: camera.view_projection().into(),
        }
    }
}
