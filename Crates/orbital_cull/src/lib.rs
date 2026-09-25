// ── CPU frustum culling infrastructure ─────────────────────────────────
//
// Frustum-sphere testing runs on the CPU. Visible instance matrices are
// compacted into `filtered_instance_buffer` (VERTEX | COPY_DST) and the
// renderer issues direct `draw_indexed` calls with per-model visible
// counts.  No GPU compute, no indirect draw buffers.

use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

/// Per-frame cull output consumed by the renderer.
///
/// Created once per device and reused each frame. The culling system calls
/// [`write_visible_instances`] each frame; the renderer reads
/// [`filtered_instance_buffer`], [`model_first_instance`], and
/// [`visible_count`].
pub struct CullResources {
    /// Vertex-read buffer containing the CPU-compacted visible instance
    /// matrices.  Layout: [model0_inst0, model0_inst1, …, model1_inst0, …]
    filtered_instance_buffer: Buffer,

    /// Per-model offset into [`Self::filtered_instance_buffer`] (in units of
    /// instances, **not** bytes).  Multiply by 64 to get the byte offset.
    first_instance: Vec<u32>,

    /// Per-model visible instance count (post-cull).
    visible_counts: Vec<u32>,
}

impl std::fmt::Debug for CullResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CullResources")
            .field("num_models", &self.visible_counts.len())
            .field("total_visible", &self.visible_counts.iter().sum::<u32>())
            .finish()
    }
}

impl CullResources {
    /// Allocate the filtered-instance buffer with enough capacity for
    /// `max_instances` mat4x4 matrices (64 bytes each).
    pub fn new(device: &Device, max_instances: u32, _max_models: u32) -> Self {
        let filtered_instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cull Filtered Instances"),
            size: max_instances as u64 * 64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            filtered_instance_buffer,
            first_instance: Vec::new(),
            visible_counts: Vec::new(),
        }
    }

    /// The vertex buffer containing the CPU-compacted visible instances.
    /// Bind at vertex-input slot 1.
    pub fn filtered_instance_buffer(&self) -> &Buffer {
        &self.filtered_instance_buffer
    }

    /// Instance offset (in instances, not bytes) for model `i` into the
    /// filtered buffer.  Multiply by 64 for the byte offset.
    pub fn model_first_instance(&self, i: usize) -> u32 {
        self.first_instance.get(i).copied().unwrap_or(0)
    }

    /// Number of visible instances for model `i` after culling.
    pub fn visible_count(&self, i: usize) -> u32 {
        self.visible_counts.get(i).copied().unwrap_or(0)
    }

    /// Upload the CPU-culled visible instances and per-model metadata.
    ///
    /// * `instances` — contiguous mat4x4 matrices (64 bytes each) of all
    ///   visible instances across all models, in model order.
    /// * `offsets` — per-model first-instance offset (in instances).
    /// * `counts` — per-model visible instance count.
    pub fn write_visible_instances(
        &mut self,
        queue: &Queue,
        instances: &[u8],
        offsets: Vec<u32>,
        counts: Vec<u32>,
    ) {
        let bytes = (self.filtered_instance_buffer.size() as usize).min(instances.len());
        if bytes > 0 {
            queue.write_buffer(&self.filtered_instance_buffer, 0, &instances[..bytes]);
        }
        self.first_instance = offsets;
        self.visible_counts = counts;
    }
}
