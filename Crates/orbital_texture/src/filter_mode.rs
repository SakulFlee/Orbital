use wgpu::{FilterMode as WFilterMode, MipmapFilterMode};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct FilterMode {
    pub mag: WFilterMode,
    pub min: WFilterMode,
    pub mipmap: MipmapFilterMode,
}

impl FilterMode {
    pub fn linear() -> Self {
        Self {
            mag: WFilterMode::Linear,
            min: WFilterMode::Linear,
            mipmap: MipmapFilterMode::Linear,
        }
    }

    pub fn nearest() -> Self {
        Self {
            mag: WFilterMode::Nearest,
            min: WFilterMode::Nearest,
            mipmap: MipmapFilterMode::Nearest,
        }
    }
}

impl Default for FilterMode {
    fn default() -> Self {
        Self {
            mag: WFilterMode::Linear,
            min: WFilterMode::Linear,
            mipmap: MipmapFilterMode::Nearest,
        }
    }
}
