use winit::dpi::{PhysicalSize, Size};

#[derive(Debug)]
pub struct WindowSettings {
    size: Size,
}

impl WindowSettings {
    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn set_size<S>(&mut self, size: S)
    where
        S: Into<Size>,
    {
        self.size = size.into();
    }
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            size: PhysicalSize::new(1280, 720).into(),
        }
    }
}
