use cgmath::Vector3;

pub const MAX_SHADOW_SLOTS: u32 = 16;
pub const SHADOW_TYPE_DIRECTIONAL_CASCADE: u32 = 0;
pub const SHADOW_TYPE_SPOT: u32 = 1;
pub const SHADOW_TYPE_POINT: u32 = 2;
pub const DEFAULT_SHADOW_RESOLUTION: u32 = 1024;
pub const DEFAULT_CASCADE_COUNT: u32 = 4;
pub const DEFAULT_CASCADE_SPLIT_LAMBDA: f32 = 0.75;
/// Default shadow bias — matches Blender's scale (~0.001). Scenes can
/// override via ShadowCaster.bias. The bias is used both for the
/// depth-comparison offset (`slope_scaled_bias` in the fragment shader)
/// and for the world-space normal‑bias offset that prevents self‑shadowing.
pub const DEFAULT_SHADOW_BIAS: f32 = 0.0005;

/// Per-slot GPU data (96 bytes, matches WGSL ShadowSlot).
/// 16 slots × 96 bytes + 16 bytes header = 1552 bytes uniform buffer.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ShadowSlotData {
    pub light_view_proj: [[f32; 4]; 4], // offset 0,   size 64
    pub shadow_type: u32,               // offset 64,  size 4
    pub layer_index: u32,               // offset 68,  size 4
    pub cascade_split_depth: f32,       // offset 72,  size 4
    pub bias: f32,                      // offset 76,  size 4
    pub light_index: u32,               // offset 80,  size 4
    pub near_plane: f32,                // offset 84,  size 4
    pub _padding: [u32; 2],             // offset 88,  size 8
} // total: 96 bytes

impl ShadowSlotData {
    pub fn as_bytes(&self) -> &[u8] {
        let size = std::mem::size_of::<Self>();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }
}

/// Uniform buffer payload: slots array + header.
/// WGSL struct ShadowData { slots: array<ShadowSlot, 16>, cascade_count: u32 }
/// Total: 96*16 + 4 → padded to 16 → 1552 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ShadowGpuData {
    pub slots: [ShadowSlotData; MAX_SHADOW_SLOTS as usize], // offset 0, size 1536
    pub cascade_count: u32,                                 // offset 1536, size 4
    pub _padding: [u32; 3],                                 // offset 1540, size 12
} // total: 1552 bytes

impl ShadowGpuData {
    pub fn new() -> Self {
        Self {
            slots: [ShadowSlotData {
                light_view_proj: [[0.0; 4]; 4],
                shadow_type: 0,
                layer_index: 0,
                cascade_split_depth: 0.0,
                bias: 0.0,
                light_index: 0,
                near_plane: 0.1,
                _padding: [0; 2],
            }; MAX_SHADOW_SLOTS as usize],
            cascade_count: 0,
            _padding: [0; 3],
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        let size = std::mem::size_of::<Self>();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    /// Number of shadow slots currently in use.
    pub fn active_slot_count(&self) -> u32 {
        self.cascade_count
    }

    /// Compute total layer count needed across all slots.
    pub fn total_layers(&self) -> u32 {
        let mut max_layer = 0u32;
        for i in 0..self.cascade_count as usize {
            if self.slots[i].layer_index + 1 > max_layer {
                max_layer = self.slots[i].layer_index + 1;
            }
        }
        max_layer
    }
}

/// ECS component: attached to a light entity to make it cast shadows.
#[derive(Debug, Clone)]
pub struct ShadowCaster {
    pub enabled: bool,
    pub resolution: u32,
    pub bias: f32,
    /// Number of CSM cascades (directional lights only; ignored for
    /// point and spot lights, which always use one shadow slot each).
    pub cascade_count: u32,
    /// Blend between uniform and logarithmic cascade splits (0.0–1.0).
    pub cascade_split_lambda: f32,
}

impl Default for ShadowCaster {
    fn default() -> Self {
        Self {
            enabled: true,
            resolution: DEFAULT_SHADOW_RESOLUTION,
            bias: DEFAULT_SHADOW_BIAS,
            cascade_count: DEFAULT_CASCADE_COUNT,
            cascade_split_lambda: DEFAULT_CASCADE_SPLIT_LAMBDA,
        }
    }
}

/// Per-light shadow info extracted each frame for the render pass.
pub struct ShadowLightInfo {
    pub light_type: u32,
    pub direction: Vector3<f32>,
    pub position: Vector3<f32>,
    pub caster: ShadowCaster,
    /// Spot light outer cone angle (radians). 0 for non-spot lights.
    pub outer_cone_angle: f32,
    /// Index of this light in the GPU `light_store` array.
    pub light_store_index: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_slot_size() {
        assert_eq!(std::mem::size_of::<ShadowSlotData>(), 96);
    }

    #[test]
    fn shadow_gpu_data_size() {
        assert_eq!(std::mem::size_of::<ShadowGpuData>(), 1552);
    }

    #[test]
    fn shadow_gpu_data_new_slots_zeroed() {
        let data = ShadowGpuData::new();
        assert_eq!(data.cascade_count, 0);
        assert_eq!(data.slots[0].shadow_type, 0);
        assert_eq!(data.slots[15].shadow_type, 0);
    }
}
