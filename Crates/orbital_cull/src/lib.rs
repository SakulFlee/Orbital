// ── CPU frustum culling infrastructure ─────────────────────────────────
//
// Frustum-sphere testing runs on the CPU. Visible instance matrices are
// compacted into `filtered_instance_buffer` (VERTEX | COPY_DST) and the
// renderer issues direct `draw_indexed` calls with per-model visible
// counts.  No GPU compute, no indirect draw buffers.

use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

/// Per-frame cull output consumed by the renderer.
///
/// The buffers and scratch vectors are persistent. [`begin_frame`] clears the
/// per-model lists, [`push_instance`] appends visible instance matrices and
/// [`upload`] writes them once. The renderer reads
/// [`filtered_instance_buffer`], [`model_first_instance`], and [`visible_count`].
///
/// [`begin_frame`]: CullResources::begin_frame
/// [`push_instance`]: CullResources::push_instance
/// [`upload`]: CullResources::upload
/// [`filtered_instance_buffer`]: CullResources::filtered_instance_buffer
/// [`model_first_instance`]: CullResources::model_first_instance
/// [`visible_count`]: CullResources::visible_count
pub struct CullResources {
    /// Vertex-read buffer containing the CPU-compacted visible instance
    /// matrices. Layout: [model0_inst0, model0_inst1, …, model1_inst0, …]
    filtered_instance_buffer: Buffer,

    /// Maximum number of instances the buffer can hold.
    capacity: u32,

    /// Maximum number of models the per-model lists can hold.
    model_capacity: u32,

    /// Per-model offset into [`Self::filtered_instance_buffer`] (in units of
    /// instances, **not** bytes). Multiply by 64 to get the byte offset.
    first_instance: Vec<u32>,

    /// Per-model visible instance count (post-cull).
    visible_counts: Vec<u32>,

    /// Scratch storage for the visible instance matrices before upload.
    visible_data: Vec<u8>,
}

impl std::fmt::Debug for CullResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CullResources")
            .field("num_models", &self.visible_counts.len())
            .field("total_visible", &self.visible_counts.iter().sum::<u32>())
            .field("capacity", &self.capacity)
            .field("model_capacity", &self.model_capacity)
            .finish()
    }
}

impl CullResources {
    /// Allocate persistent cull buffers with capacity for `max_instances`
    /// mat4x4 matrices (64 bytes each) and `max_models` models.
    pub fn new(device: &Device, max_instances: u32, max_models: u32) -> Self {
        let capacity = max_instances.max(1);
        let model_capacity = max_models.max(1);

        let filtered_instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cull Filtered Instances"),
            size: capacity as u64 * 64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            filtered_instance_buffer,
            capacity,
            model_capacity,
            first_instance: Vec::with_capacity(model_capacity as usize),
            visible_counts: Vec::with_capacity(model_capacity as usize),
            visible_data: Vec::with_capacity(capacity as usize * 64),
        }
    }

    /// Instance capacity of the filtered buffer.
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Model capacity of the per-model lists.
    pub fn model_capacity(&self) -> u32 {
        self.model_capacity
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

    /// Clear the per-model lists before a new cull pass.
    pub fn begin_frame(&mut self) {
        self.first_instance.clear();
        self.visible_counts.clear();
        self.visible_data.clear();
    }

    /// Append a visible instance's 64-byte matrix to the compacted list.
    pub fn push_instance(&mut self, instance: &[u8]) {
        debug_assert_eq!(instance.len(), 64);
        self.visible_data.extend_from_slice(instance);
    }

    /// Record one model's first-instance offset and visible count.
    pub fn push_model(&mut self, first_instance: u32, visible_count: u32) {
        self.first_instance.push(first_instance);
        self.visible_counts.push(visible_count);
    }

    /// Upload the compacted visible instances to the GPU.
    pub fn upload(&mut self, queue: &Queue) {
        if !self.visible_data.is_empty() {
            queue.write_buffer(&self.filtered_instance_buffer, 0, &self.visible_data);
        }
    }
}
