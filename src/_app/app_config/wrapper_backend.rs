use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum WrapperBackend {
    DX12 = 0,
    Metal = 1,
    Vulkan = 2,
    DX11 = 3,
    BrowserGPU = 4,
    OpenGL = 5,
}
