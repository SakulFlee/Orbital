use serde::{Deserialize, Serialize};
use winit::dpi::PhysicalSize;

#[derive(Serialize, Deserialize)]
pub struct ConfigWindow {
    pub size: (u32, u32),
}

impl ConfigWindow {
    pub fn to_physical_size(&self) -> PhysicalSize<u32> {
        PhysicalSize {
            width: self.size.0,
            height: self.size.1,
        }
    }
}
