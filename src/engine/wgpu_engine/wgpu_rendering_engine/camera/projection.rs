use cgmath::{perspective, Matrix4, Rad};

use super::Camera;

#[derive(Debug)]
pub struct Projection {
    width: u32,
    height: u32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            width,
            height,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.set_width(width);
        self.set_height(height);
    }

    pub fn calculate_matrix(&self) -> Matrix4<f32> {
        Camera::OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect(), self.znear, self.zfar)
    }

    pub fn aspect(&self) -> f32 {
        self.width() as f32 / self.height as f32
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
    }

    pub fn fovy(&self) -> Rad<f32> {
        self.fovy
    }

    pub fn set_fovy(&mut self, fovy: Rad<f32>) {
        self.fovy = fovy;
    }

    pub fn znear(&self) -> f32 {
        self.znear
    }

    pub fn set_znear(&mut self, znear: f32) {
        self.znear = znear;
    }

    pub fn zfar(&self) -> f32 {
        self.zfar
    }

    pub fn set_zfar(&mut self, zfar: f32) {
        self.zfar = zfar;
    }
}
