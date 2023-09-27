use cgmath::{Point3, Vector3};

pub struct CameraChange {
    eye: Option<Point3<f32>>,
    target: Option<Point3<f32>>,
    up: Option<Vector3<f32>>,
    aspect: Option<f32>,
    fovy: Option<f32>,
    znear: Option<f32>,
    zfar: Option<f32>,
}

impl CameraChange {
    pub fn new() -> Self {
        Self {
            eye: None,
            target: None,
            up: None,
            aspect: None,
            fovy: None,
            znear: None,
            zfar: None,
        }
    }

    pub fn with_eye(mut self, eye: Point3<f32>) -> Self {
        self.eye = Some(eye);
        self
    }

    pub fn with_target(mut self, target: Point3<f32>) -> Self {
        self.target = Some(target);
        self
    }

    pub fn with_up(mut self, up: Vector3<f32>) -> Self {
        self.up = Some(up);
        self
    }

    pub fn with_aspect(mut self, aspect: f32) -> Self {
        self.aspect = Some(aspect);
        self
    }

    pub fn with_fovy(mut self, fovy: f32) -> Self {
        self.fovy = Some(fovy);
        self
    }

    pub fn with_znear(mut self, znear: f32) -> Self {
        self.znear = Some(znear);
        self
    }

    pub fn with_zfar(mut self, zfar: f32) -> Self {
        self.zfar = Some(zfar);
        self
    }

    pub fn get_eye(&self) -> Option<Point3<f32>> {
        self.eye
    }

    pub fn get_target(&self) -> Option<Point3<f32>> {
        self.target
    }

    pub fn get_up(&self) -> Option<Vector3<f32>> {
        self.up
    }

    pub fn get_aspect(&self) -> Option<f32> {
        self.aspect
    }

    pub fn get_fovy(&self) -> Option<f32> {
        self.fovy
    }

    pub fn get_znear(&self) -> Option<f32> {
        self.znear
    }

    pub fn get_zfar(&self) -> Option<f32> {
        self.zfar
    }
}

impl Default for CameraChange {
    fn default() -> Self {
        Self::new()
    }
}
