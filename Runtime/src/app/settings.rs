use winit::dpi::{PhysicalSize, Size};

pub struct RuntimeSettings {
    pub name: String,
    pub size: Size,
    pub vsync_enabled: bool,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            name: "Default App".into(),
            size: PhysicalSize::new(1280, 720).into(),
            vsync_enabled: true,
        }
    }
}
