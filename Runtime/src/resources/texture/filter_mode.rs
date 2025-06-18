use wgpu::FilterMode as WFilterMode;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct FilterMode {
    pub mag: WFilterMode,
    pub min: WFilterMode,
    pub mipmap: WFilterMode,
}

impl FilterMode {
    pub fn linear() -> Self {
        Self {
            mag: WFilterMode::Linear,
            min: WFilterMode::Linear,
            mipmap: WFilterMode::Linear,
        }
    }

    pub fn nearest() -> Self {
        Self {
            mag: WFilterMode::Nearest,
            min: WFilterMode::Nearest,
            mipmap: WFilterMode::Nearest,
        }
    }
}

impl Default for FilterMode {
    fn default() -> Self {
        Self {
            mag: WFilterMode::Linear,
            min: WFilterMode::Linear,
            mipmap: WFilterMode::Nearest,
        }
    }
}
