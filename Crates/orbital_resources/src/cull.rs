// ── GPU frustum culling infrastructure ─────────────────────────────────

use crate::Frustum;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    ComputePipeline, ComputePipelineDescriptor, Device, PipelineLayoutDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, util::DeviceExt,
};

/// Runtime cull shader — the single source of truth, shared with the naga
/// validation test (`tests/wgsl_shaders.rs`) and the standalone WGSL file.
const SHADER: &str = include_str!("../../../Assets/Shaders/instance_cull.wgsl");

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
    cull_all_pipeline: ComputePipeline,
    finalize_pipeline: ComputePipeline,
    bind_group: BindGroup,

    debug_cull_all: bool,
    debug_single_encoder: bool,

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
        Self::with_debug(device, max_instances, max_models, false, false)
    }

    /// Like [`new`], but with `debug_cull_all = true` the `cull_all` entry
    /// point is also compiled (used by `ORBITAL_CULL_DEBUG=cull_all` probing —
    /// same compaction/indirect path as `cull`, but no frustum test).
    ///
    /// `debug_single_encoder = true` marks this resource for single-encoder
    /// culling: the compute dispatch is submitted together with the render
    /// pass (see [`Self::dispatch_into_render`]) instead of in a separate
    /// submission. Used by `ORBITAL_CULL_SINGLE_ENCODER=1` to sidestep
    /// cross-submission storage→vertex/indirect barrier gaps on some drivers.
    pub fn with_debug(
        device: &Device,
        max_instances: u32,
        max_models: u32,
        debug_cull_all: bool,
        debug_single_encoder: bool,
    ) -> Self {
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

        // Optional debug pipeline — same compaction/indirect path as `cull`
        // but without the frustum test (ORBITAL_CULL_DEBUG=cull_all).
        let cull_all_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Cull All Pass"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: if debug_cull_all { Some("cull_all") } else { None },
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
        let params_size = max_models as u64 * 32; // 8 × u32 per model (two vec4<u32>)
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
            cull_all_pipeline,
            finalize_pipeline,
            bind_group,
            debug_cull_all,
            debug_single_encoder,
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

    /// Upload per-model params (8 × u32 = 32 B per model:
    /// `[first_instance, total_count, index_count, first_index,
    /// base_vertex, pad, pad, pad]` × `num_models`).
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

    /// Whether this resource is in single-encoder mode (compute culled inside
    /// the render submission, see [`Self::dispatch_into_render`]).
    pub fn single_encoder(&self) -> bool {
        self.debug_single_encoder
    }

    /// Whether the `cull_all` (frustum-test-skipping) entry point is active.
    pub fn cull_all(&self) -> bool {
        self.debug_cull_all
    }

    /// Dispatch both cull passes into `encoder` — used by the renderer when in
    /// single-encoder mode so the storage→vertex/indirect buffer transitions
    /// are tracked within one submission. Workgroup sizing is safe: X is
    /// rounded up over the *total* instance count (any extra instances
    /// early-return via `local_idx >= instance_count`), Y is the model count.
    pub fn dispatch_into_render(&self, encoder: &mut wgpu::CommandEncoder) {
        self.dispatch(
            encoder,
            self.max_models,
            self.max_instances,
            self.debug_cull_all,
        );
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
    ///
    /// With `cull_all = true` (debug probing via `ORBITAL_CULL_DEBUG=cull_all`),
    /// pass 1 uses the `cull_all` entry point, which admits every instance
    /// without the frustum test — compaction, counters and indirect args are
    /// handled identically.
    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        num_models: u32,
        max_inst_per_model: u32,
        cull_all: bool,
    ) {
        // Pass 1 — cull
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Cull Compute Pass"),
                timestamp_writes: None,
            });
            if cull_all {
                pass.set_pipeline(&self.cull_all_pipeline);
            } else {
                pass.set_pipeline(&self.cull_pipeline);
            }
            pass.set_bind_group(0, &self.bind_group, &[]);
            // Dispatch: X = instances per model (rounded up to workgroup size),
            //          Y = number of models
            let wg_x = max_inst_per_model.div_ceil(64);
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

    /// Debug readback of per-model counters + indirect args.
    ///
    /// Must be called **after** `dispatch`'s submission has been enqueued.
    /// Blocks until the GPU work is done, so only use it while debugging
    /// (`ORBITAL_CULL_DEBUG=1` / `=cull_all`)!
    pub fn readback_cull_state(&self, device: &Device, queue: &wgpu::Queue, num_models: u32) {
        if num_models == 0 {
            return;
        }
        let counters_bytes = (num_models as u64 * 4).max(4);
        let indirect_bytes = num_models as u64 * 20;

        let staging = device.create_buffer(&BufferDescriptor {
            label: Some("Cull Debug Staging"),
            size: counters_bytes.max(indirect_bytes),
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Cull Debug Readback"),
        });
        enc.copy_buffer_to_buffer(&self.counters_buffer, 0, &staging, 0, counters_bytes);
        enc.copy_buffer_to_buffer(&self.indirect_buffer, 0, &staging, 0, indirect_bytes);
        queue.submit(Some(enc.finish()));

        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let done2 = done.clone();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |_| {
            done2.store(true, std::sync::atomic::Ordering::Relaxed);
        });
        let mapped = poll_until(
            device,
            &done,
            std::time::Duration::from_millis(500),
        );

        if mapped
            && let Ok(data) = staging.slice(..).get_mapped_range()
        {
            log::debug!("──────── ORBITAL_CULL_DEBUG readback ────────");
            for m in 0..num_models as usize {
                let cb: [u8; 4] = data[m * 4..m * 4 + 4].try_into().unwrap();
                let visible = u32::from_le_bytes(cb);
                let ib: [u8; 20] = data[m * 20..m * 20 + 20].try_into().unwrap();
                let index_count = u32::from_le_bytes(ib[0..4].try_into().unwrap());
                let instance_count = u32::from_le_bytes(ib[4..8].try_into().unwrap());
                let first_index = u32::from_le_bytes(ib[8..12].try_into().unwrap());
                let base_vertex = i32::from_le_bytes(ib[12..16].try_into().unwrap());
                let first_instance = u32::from_le_bytes(ib[16..20].try_into().unwrap());
                log::debug!(
                    "model {m}: visible={visible}  indirect=[index_count={index_count}, instance_count={instance_count}, first_index={first_index}, base_vertex={base_vertex}, first_instance={first_instance}]"
                );
            }
            log::debug!("─────────────────────────────────────────────");
        } else {
            log::warn!("ORBITAL_CULL_DEBUG: staging buffer not ready after poll");
        }
        staging.unmap();
    }
}

/// Busy-polls the device until `done` is set (map callback fired) or the
/// timeout elapses. Returns `true` when the buffer is ready to read.
fn poll_until(
    device: &Device,
    done: &std::sync::atomic::AtomicBool,
    timeout: std::time::Duration,
) -> bool {
    use std::sync::atomic::Ordering;
    let start = std::time::Instant::now();
    while !done.load(Ordering::Relaxed) {
        match device.poll(wgpu::PollType::Poll) {
            Ok(_) => {}
            Err(e) => log::warn!("Cull debug readback poll error: {e:?}"),
        }
        if done.load(Ordering::Relaxed) {
            return true;
        }
        if start.elapsed() > timeout {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    true
}
