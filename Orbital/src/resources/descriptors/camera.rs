use cgmath::Point3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraDescriptor {
    pub position: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fovy: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for CameraDescriptor {
    fn default() -> Self {
        Self {
            // position: Point3::new(0.0, 4.0, -2.5),
            position: Point3::new(-1.0, 0.0, 0.0),
            yaw: 0f32,
            pitch: 0f32,
            // 16:9 aspect ratio as default
            aspect: 16.0 / 9.0,
            // fovy: PI / 4.0, TODO
            fovy: 45.0,
            near: 0.1,
            far: 10000.0,
        }
    }
}
