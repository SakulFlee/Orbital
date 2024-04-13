use winit::dpi::{PhysicalSize, Size};

pub struct RuntimeSettings {
    pub name: String,
    pub size: Size,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            name: "Default App".into(),
            size: PhysicalSize::new(1280, 720).into(),
        }
    }
}
