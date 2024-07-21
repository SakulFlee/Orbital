use cgmath::Point3;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraDescriptor {
    pub identifier: String,
    pub position: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fovy: f32,
    pub near: f32,
    pub far: f32,
}

impl CameraDescriptor {
    pub const DEFAULT_NAME: &'static str = "Default";
}

impl Default for CameraDescriptor {
    fn default() -> Self {
        Self {
            identifier: Self::DEFAULT_NAME.into(),
            position: Point3::new(-1.0, 0.0, 0.0),
            yaw: 0f32,
            pitch: 0f32,
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            near: 0.1,
            far: 10000.0,
        }
    }
}
