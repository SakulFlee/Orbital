// ── GPU frustum culling infrastructure ─────────────────────────────────

use crate::Frustum;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    ComputePipeline, ComputePipelineDescriptor, Device, PipelineLayoutDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, util::DeviceExt,
};

const SHADER: &str = r#"
struct ModelParams {
    first_instance: u32,
    total_count: u32,
    index_count: u32,
    first_index: u32,
    base_vertex: i32,
    _pad: u32,
};

struct DrawIndexedIndirect {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

@group(0) @binding(0) var<uniform> frustum_planes: array<vec4<f32>, 6>;
@group(0) @binding(1) var<storage, read> params: array<ModelParams>;
@group(0) @binding(2) var<storage, read> in_instances: array<mat4x4<f32>>;
@group(0) @binding(3) var<storage, read> in_bounds: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> out_instances: array<mat4x4<f32>>;
@group(0) @binding(5) var<storage, read_write> counters: array<atomic<u32>>;
@group(0) @binding(6) var<storage, read_write> indirect: array<DrawIndexedIndirect>;

@compute @workgroup_size(64)
fn cull(@builtin(global_invocation_id) id: vec3<u32>) {
    let model = id.y;
    let local_idx = id.x;
    let p = params[model];
    if local_idx >= p.total_count { return; }

    let global_idx = p.first_instance + local_idx;
    let bounds = in_bounds[global_idx];

    for (var i = 0u; i < 6u; i++) {
        let plane = frustum_planes[i];
        let dist = dot(plane.xyz, bounds.xyz) + plane.w;
        if dist <= -bounds.w { return; }
    }

    let slot = atomicAdd(&counters[model], 1u);
    out_instances[p.first_instance + slot] = in_instances[global_idx];
}

@compute @workgroup_size(1)
fn finalize(@builtin(global_invocation_id) id: vec3<u32>) {
    let model = id.x;
    let p = params[model];
    let count = atomicExchange(&counters[model], 0u);

    indirect[model] = DrawIndexedIndirect(
        p.index_count,
        count,
        p.first_index,
        p.base_vertex,
        0u,
    );
}
"#;

/// All GPU resources needed for compute‑based per‑instance culling.
///
/// Created once per device and reused each frame. The culling system
/// calls [`upload_frustum`], [`upload_instances_and_bounds`], [`upload_params`],
/// and [`dispatch`] each frame. The renderer reads [`indirect_buffer`] and
/// [`compacted_buffer`] for indirect draws.
pub struct CullResources {
    max_instances: u32,
    max_models: u32,

    pub frustum_buffer: Buffer,
    pub params_buffer: Buffer,
    pub instances_buffer: Buffer,
    pub bounds_buffer: Buffer,
    pub compacted_buffer: Buffer,
    pub counters_buffer: Buffer,
    pub indirect_buffer: Buffer,

    cull_pipeline: ComputePipeline,
    finalize_pipeline: ComputePipeline,
    bind_group: BindGroup,

    first_instance: Vec<u32>,
}

impl std::fmt::Debug for CullResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CullResources")
            .field("max_instances", &self.max_instances)
            .field("max_models", &self.max_models)
            .field("num_models", &self.first_instance.len())
            .finish()
    }
}

impl CullResources {
    pub fn new(device: &Device, max_instances: u32, max_models: u32) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Cull Compute Shader"),
            source: ShaderSource::Wgsl(SHADER.into()),
        });

        // ── Bind group layout ─────────────────────────────────────────
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Cull BindGroup Layout"),
            entries: &[
                // 0 — frustum planes (uniform)
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 1 — per-model params
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 2 — input instance matrices
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 3 — input instance bounds
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 4 — output compacted instances
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 5 — per-model counters (atomic)
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 6 — indirect draw args
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Cull Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let cull_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Cull Pass"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cull"),
            compilation_options: Default::default(),
            cache: None,
        });

        let finalize_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Finalize Pass"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("finalize"),
            compilation_options: Default::default(),
            cache: None,
        });

        // ── Buffers ────────────────────────────────────────────────────
        let frustum_size = 6 * 4 * 4; // 6 × vec4<f32> = 96 bytes
        let params_size = max_models as u64 * 24; // 6 × u32 per model
        let instances_size = max_instances as u64 * 64; // mat4x4<f32> each
        let bounds_size = max_instances as u64 * 16; // vec4<f32> each
        let counters_size = max_models as u64 * 4;
        let indirect_entry_size = 20u64; // DrawIndexedIndirect

        let frustum_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cull Frustum"),
            size: frustum_size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let params_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cull Params"),
            size: params_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instances_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cull In Instances"),
            size: instances_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bounds_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cull In Bounds"),
            size: bounds_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let compacted_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cull Compacted Out"),
            size: instances_size,
            usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let counters_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cull Counters"),
            contents: &vec![0u8; counters_size as usize],
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });
        let indirect_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cull Indirect Args"),
            size: max_models as u64 * indirect_entry_size,
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Cull BindGroup"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: frustum_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: instances_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: bounds_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: compacted_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: counters_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: indirect_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            max_instances,
            max_models,
            frustum_buffer,
            params_buffer,
            instances_buffer,
            bounds_buffer,
            compacted_buffer,
            counters_buffer,
            indirect_buffer,
            cull_pipeline,
            finalize_pipeline,
            bind_group,
            first_instance: Vec::new(),
        }
    }

    /// Upload 6 frustum planes (each as `[nx, ny, nz, d]`).
    pub fn upload_frustum(&self, queue: &wgpu::Queue, frustum: &Frustum) {
        let mut data = [0u8; 96];
        for (i, plane) in frustum.planes.iter().enumerate() {
            let base = i * 16;
            data[base..base + 4].copy_from_slice(&plane.normal.x.to_le_bytes());
            data[base + 4..base + 8].copy_from_slice(&plane.normal.y.to_le_bytes());
            data[base + 8..base + 12].copy_from_slice(&plane.normal.z.to_le_bytes());
            data[base + 12..base + 16].copy_from_slice(&plane.d.to_le_bytes());
        }
        queue.write_buffer(&self.frustum_buffer, 0, &data);
    }

    /// Upload per-model params (`[first_instance, total_count, index_count,
    /// first_index, base_vertex, _pad]` × `num_models`).
    pub fn upload_params(&self, queue: &wgpu::Queue, params_bytes: &[u8]) {
        queue.write_buffer(&self.params_buffer, 0, params_bytes);
    }

    /// Store the first_instance offsets for each model (CPU side, used by
    /// the renderer to compute vertex-buffer binding offsets).
    pub fn set_model_offsets(&mut self, offsets: Vec<u32>) {
        self.first_instance = offsets;
    }

    /// First-instance offset for model `i` — in units of instances, **not**
    /// bytes. Multiply by 64 to get the byte offset into the compacted buffer.
    pub fn max_instances(&self) -> u32 {
        self.max_instances
    }
    pub fn max_models(&self) -> u32 {
        self.max_models
    }
    pub fn counters_buffer(&self) -> &Buffer {
        &self.counters_buffer
    }
    pub fn compacted_buffer(&self) -> &Buffer {
        &self.compacted_buffer
    }
    pub fn indirect_buffer(&self) -> &Buffer {
        &self.indirect_buffer
    }

    pub fn model_first_instance(&self, i: usize) -> u32 {
        self.first_instance.get(i).copied().unwrap_or(0)
    }

    /// Upload all instance matrices (flat) and bounds (flat).
    pub fn upload_instances_and_bounds(
        &self,
        queue: &wgpu::Queue,
        instance_data: &[u8],
        bounds_data: &[u8],
    ) {
        queue.write_buffer(&self.instances_buffer, 0, instance_data);
        queue.write_buffer(&self.bounds_buffer, 0, bounds_data);
    }

    /// Dispatch the cull and finalize compute passes.
    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        num_models: u32,
        max_inst_per_model: u32,
    ) {
        // Pass 1 — cull
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Cull Compute Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.cull_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            // Dispatch: X = instances per model (rounded up to workgroup size),
            //          Y = number of models
            let wg_x = (max_inst_per_model + 63) / 64;
            pass.dispatch_workgroups(wg_x.max(1), num_models.max(1), 1);
        }

        // Pass 2 — finalize
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Cull Finalize Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.finalize_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(num_models.max(1), 1, 1);
        }
    }
}
