/// Per-model visibility data produced by the frustum-culling system.
///
/// The renderer uses this instead of the model's built-in instance buffer
/// when the data is provided.
#[derive(Clone, Debug)]
pub struct CullModelInfo {
    /// Number of visible instances for this model.
    ///
    /// When zero the renderer issues a no‑op draw.
    pub visible_count: u32,
    /// GPU buffer containing only the visible instance matrices.
    ///
    /// Layout matches the instancing attributes (4 × `vec4<f32>` per instance).
    pub visible_buffer: wgpu::Buffer,
}

impl CullModelInfo {
    pub fn empty(device: &wgpu::Device) -> Self {
        Self {
            visible_count: 0,
            visible_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("CullModelInfo (empty)"),
                size: 64,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            }),
        }
    }
}
