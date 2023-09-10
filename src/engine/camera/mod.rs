use cgmath::{perspective, Deg, Matrix4, Point3, Vector3};

pub mod camera_uniform;
pub use camera_uniform::*;

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    #[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
    );

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view_matrix = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let projection_matrix = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return Camera::OPENGL_TO_WGPU_MATRIX * projection_matrix * view_matrix;
    }
}