use std::any::Any;
use std::sync::{Arc, Mutex};

pub struct IcedState {
    data: Arc<Mutex<dyn Any + Send>>,
    font_size: f32,
    screen_size: (f32, f32),
}

impl Default for IcedState {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(())),
            font_size: 16.0,
            screen_size: (800.0, 600.0),
        }
    }
}

impl IcedState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_data<T: Send + 'static>(data: T) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
            ..Default::default()
        }
    }

    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_size = (width, height);
    }

    pub fn screen_size(&self) -> (f32, f32) {
        self.screen_size
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn data(&self) -> Arc<Mutex<dyn Any + Send>> {
        self.data.clone()
    }
}
