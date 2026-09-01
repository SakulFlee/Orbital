struct ShadowSlot {
    light_view_proj: mat4x4<f32>,
    shadow_type: u32,
    layer_index: u32,
    cascade_split_depth: f32,
    bias: f32,
    light_index: u32,
    near_plane: f32,
}

struct ShadowData {
    slots: array<ShadowSlot, 16>,
    cascade_count: u32,
}
