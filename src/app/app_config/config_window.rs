use serde::{Deserialize, Serialize};
use winit::dpi::PhysicalSize;

#[derive(Serialize, Deserialize)]
pub struct ConfigWindow {
    pub width: u32,
    pub height: u32,
}

impl ConfigWindow {
    pub fn to_physical_size(&self) -> PhysicalSize<u32> {
        PhysicalSize {
            width: self.width,
            height: self.height,
        }
    }
}
