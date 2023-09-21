use serde::{Deserialize, Serialize};
use winit::dpi::{PhysicalPosition, PhysicalSize};

use super::wrapper_fullscreen::WrapperFullscreen;

#[derive(Serialize, Deserialize)]
pub struct ConfigMonitor {
    pub fullscreen: WrapperFullscreen,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub refresh_rate: u32,
}

impl ConfigMonitor {
    pub fn to_physical_position(&self) -> PhysicalPosition<i32> {
        PhysicalPosition {
            x: self.position.0,
            y: self.position.1,
        }
    }

    pub fn to_physical_size(&self) -> PhysicalSize<u32> {
        PhysicalSize {
            width: self.size.0,
            height: self.size.1,
        }
    }
}
