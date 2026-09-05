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
    /// Compute-write target (STORAGE): visible instances are compacted here.
    pub compacted_buffer: Buffer,
    /// Vertex-read source (VERTEX | COPY_DST): the compute's compacted output is
    /// copied into this buffer before drawing. Breaking storage→vertex this way
    /// sidesteps the Adreno weak spot of reading a compute-written storage
    /// buffer directly as vertex data.
    pub compacted_vertex_buffer: Buffer,
    pub counters_buffer: Buffer,
    /// Compute-write target (STORAGE): `finalize` writes the per-model
    /// DrawIndexedIndirect args here; they are then copied into
    /// [`Self::indirect_buffer`] before drawing — mirroring the
    /// compacted→vertex copy, so the `INDIRECT` bind point is never fed by a
    /// compute-written storage buffer either.
    pub indirect_storage_buffer: Buffer,
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
    /// Probe helper: enqueue a copy of `byte_len` bytes from `src` into the
    /// debug readback's staging region at `dst_offset`. Lets
    /// `readback_cull_state` dump *both sides* of each intermediary copy
    /// (storage vs consumer buffer) to tell "compute stored nothing" apart
    /// from "the copy dropped it".
    fn probe_copy(
        &self,
        enc: &mut wgpu::CommandEncoder,
        staging: &Buffer,
        src: &Buffer,
        dst_offset: u64,
        byte_len: u64,
    ) {
        enc.copy_buffer_to_buffer(src, 0, staging, dst_offset, byte_len);
    }

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

        // Always compile the `cull_all` entry point too (same compaction /
        // indirect path as `cull`, but without the frustum test). Compiling it
        // unconditionally means enabling the `cull_all` probe does NOT require
        // a resource reallocation — the probe exercises the identical
        // single-allocation path as production culling (a clean control).
        let cull_all_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Cull All Pass"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cull_all"),
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
        // Compute writes here (STORAGE); the contents are then copied to
        // `compacted_vertex_buffer` before drawing (see `dispatch`). Not read
        // as vertex data directly.
        let compacted_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cull Compacted Out (Storage)"),
            size: instances_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        // The actual vertex source — filled by a COPY from `compacted_buffer`
        // each frame after the compute dispatch, avoiding a compute-write →
        // vertex-read alias on drivers that mishandle it.
        let compacted_vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cull Compacted Out (Vertex)"),
            size: instances_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::COPY_SRC, // readback src
            mapped_at_creation: false,
        });
        let counters_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cull Counters"),
            contents: &vec![0u8; counters_size as usize],
            usage: BufferUsages::STORAGE
                | BufferUsages::COPY_DST
                | BufferUsages::COPY_SRC, // readback: source for copy to staging
        });
        // Compute writes the DrawIndexedIndirect args here (STORAGE); they are
        // then copied into `indirect_buffer` before drawing. Same rationale as
        // the compacted→vertex copy: never consume a compute-written STORAGE
        // buffer directly, whether as vertex data or as indirect args.
        let indirect_storage_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cull Indirect Args (Storage)"),
            size: max_models as u64 * indirect_entry_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let indirect_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cull Indirect Args (Indirect)"),
            size: max_models as u64 * indirect_entry_size,
            usage: BufferUsages::INDIRECT
                | BufferUsages::COPY_DST
                | BufferUsages::COPY_SRC, // readback: source for copy to staging
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
                    resource: indirect_storage_buffer.as_entire_binding(),
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
            compacted_vertex_buffer,
            counters_buffer,
            indirect_storage_buffer,
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
    /// The vertex-read buffer containing the compacted visible instances
    /// (filled by a copy from [`Self::compacted_buffer`] after each dispatch).
    /// This is the buffer the renderer should bind at vertex-input slot 1.
    pub fn compacted_vertex_buffer(&self) -> &Buffer {
        &self.compacted_vertex_buffer
    }
    pub fn indirect_buffer(&self) -> &Buffer {
        &self.indirect_buffer
    }
    /// The compute-write target for the per-model indirect draw args. The
    /// render pass never reads this directly — it consumes
    /// [`Self::indirect_buffer`], which is filled by a copy after each
    /// dispatch.
    pub fn indirect_storage_buffer(&self) -> &Buffer {
        &self.indirect_storage_buffer
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

    /// Set the `cull_all` (frustum-test-skipping) runtime mode. Because both
    /// entry points are always compiled, this can be toggled without a
    /// resource reallocation — each frame picks the pipeline at dispatch time.
    pub fn set_cull_all(&mut self, v: bool) {
        self.debug_cull_all = v;
    }

    /// Set single-encoder mode (compute culled inside the render submission).
    pub fn set_single_encoder(&mut self, v: bool) {
        self.debug_single_encoder = v;
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

        self.copy_compacted_to_vertex(encoder);
        self.copy_indirect_args(encoder);
    }

    /// Stage the compute's compacted instance buffer into the vertex-read buffer
    /// so the render pass consumes a plain VERTEX bind point rather than reading
    /// a compute-written STORAGE buffer directly (which Adreno mishandles).
    fn copy_compacted_to_vertex(&self, encoder: &mut wgpu::CommandEncoder) {
        let bytes = self.max_instances as u64 * 64; // 64 B per instance (mat4)
        encoder.copy_buffer_to_buffer(
            &self.compacted_buffer,
            0,
            &self.compacted_vertex_buffer,
            0,
            bytes,
        );
    }

    /// Stage the finalize pass' per-model indirect draw args into the
    /// `INDIRECT`-usage buffer the render pass consumes, mirroring
    /// [`Self::copy_compacted_to_vertex`]: the `draw_indexed_indirect` args
    /// buffer is never a compute-written storage buffer.
    fn copy_indirect_args(&self, encoder: &mut wgpu::CommandEncoder) {
        let bytes = self.max_models as u64 * 20; // 20 B per DrawIndexedIndirect
        encoder.copy_buffer_to_buffer(
            &self.indirect_storage_buffer,
            0,
            &self.indirect_buffer,
            0,
            bytes,
        );
    }

    /// CPU-args probe (`ORBITAL_CULL_CPU_ARGS` / `orbital_cull_cpu_args`):
    /// bypass the cull compute entirely — write known-correct indirect draw
    /// args and "compacted" instance matrices from the CPU.
    ///
    /// Args: each model draws its full (un-culled) instance count; the first
    /// model therefore always renders, which discriminates "the driver does
    /// not execute `draw_indexed_indirect` with compute-written args" from
    /// "the draw works and the GPU-computed data itself is the problem".
    /// The compacted instance data is filled with the same matrices that were
    /// uploaded as cull input (a manual compaction admitting everything), so
    /// vertex data stays valid too. Counters are never touched: no compute
    /// runs in this mode, so they stay zero (buffers are zero-initialised).
    pub fn write_cpu_args(
        &self,
        queue: &wgpu::Queue,
        // (index_count, instance_count, first_index, base_vertex, first_instance)
        model_params: &[(u32, u32, u32, i32, u32)],
        instance_matrices: &[u8],
    ) {
        // Indirect args: (index_count, instance_count, first_index,
        // base_vertex, first_instance) — instance_count = full model count,
        // first_instance = 0 (compacted layout == input layout here).
        let mut args = Vec::with_capacity(model_params.len() * 20);
        for &(index_count, instance_count, first_index, base_vertex, _first_instance) in
            model_params
        {
            args.extend_from_slice(&index_count.to_le_bytes());
            args.extend_from_slice(&instance_count.to_le_bytes());
            args.extend_from_slice(&first_index.to_le_bytes());
            args.extend_from_slice(&(base_vertex as u32).to_le_bytes());
            args.extend_from_slice(&0u32.to_le_bytes()); // first_instance
        }
        queue.write_buffer(&self.indirect_buffer, 0, &args);

        // Compacted instances: identity compaction (same order as input).
        let bytes = (self.max_instances as u64 * 64).min(instance_matrices.len() as u64);
        queue.write_buffer(&self.compacted_vertex_buffer, 0, &instance_matrices[..bytes as usize]);

        // Readback sentinel in the first instance matrix: CPU wrote it this
        // frame, so a probe-mode matrix dump of [1.5, 2.5, 3.5, ...] confirms
        // the write landed (and that the readback pipeline itself works).
        if bytes >= 16 {
            // Near-identity scale (0.1–0.3% off): visually harmless when the
            // sentinel instance is drawn, but unmistakable in the dump (a real
            // transform carries rotation/translation values here).
            let m0: [f32; 16] = [
                1.001, 0.0, 0.0, 0.0, 0.0, 1.002, 0.0, 0.0, 0.0, 0.0, 1.003, 0.0, 0.0, 0.0, 0.0,
                1.0,
            ];
            let mut m0b = Vec::with_capacity(64);
            for v in m0 {
                m0b.extend_from_slice(&v.to_le_bytes());
            }
            queue.write_buffer(&self.compacted_vertex_buffer, 0, &m0b);
        }
        log::info!(
            "cull CPU-args probe: wrote {} models (model0 index_count={}) + {} instance bytes",
            model_params.len(),
            model_params.first().map(|p| p.0).unwrap_or(0),
            bytes,
        );
    }

    /// Debug readback of per-model counters + indirect args.
    ///
    /// Must be called **after** `dispatch`'s submission has been enqueued.
    /// Blocks until the GPU work is done, so only use it while debugging
    /// (`ORBITAL_CULL_DEBUG=1` / `=cull_all`)!
    ///
    /// Dumps *both sides* of each intermediary copy (storage vs consumer) so a
    /// zero in the draw-visible buffer can be attributed to the compute store
    /// or to the copy. Also self-identifies which debug mode was active, and
    /// reports the CPU-args sentinel ([1.5, 2.5, 3.5, 4.5, 5.5, ...]) when the
    /// CPU-args probe wrote the data that frame.
    pub fn readback_cull_state(&self, device: &Device, queue: &wgpu::Queue, num_models: u32) {
        if num_models == 0 {
            return;
        }
        let cpu_args = orbital_core::debug_flags::cull_cpu_args();
        let counters_bytes = (num_models as u64 * 4).max(4);
        let indirect_bytes = num_models as u64 * 20;
        let matrix_bytes = 64u64;

        // Staging regions: [counters | indirect args | matrix], then one
        // trailing probe region per intermediary copy (args-storage side,
        // matrix-storage side), each of the same size, aligned to their start.
        let args_probe_off = counters_bytes + indirect_bytes + matrix_bytes;
        let mat_probe_off = args_probe_off + indirect_bytes;
        let total = mat_probe_off + matrix_bytes;
        let staging = device.create_buffer(&BufferDescriptor {
            label: Some("Cull Debug Staging"),
            size: total,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Cull Debug Readback"),
        });
        enc.copy_buffer_to_buffer(&self.counters_buffer, 0, &staging, 0, counters_bytes);
        // Consumer side (what the draw consumes) + storage side (what finalize
        // wrote) of the args intermediary copy.
        enc.copy_buffer_to_buffer(&self.indirect_buffer, 0, &staging, counters_bytes, indirect_bytes);
        self.probe_copy(&mut enc, &staging, &self.indirect_storage_buffer, args_probe_off, indirect_bytes);
        // Consumer + storage side of the instance-matrix intermediary copy.
        enc.copy_buffer_to_buffer(
            &self.compacted_vertex_buffer,
            0,
            &staging,
            args_probe_off,
            matrix_bytes,
        );
        self.probe_copy(&mut enc, &staging, &self.compacted_buffer, mat_probe_off, matrix_bytes);
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
            log::debug!(
                "──────── ORBITAL_CULL_DEBUG readback (cpu_args={cpu_args}) ────────"
            );
            let u32_at = |o: usize| -> u32 {
                u32::from_le_bytes(data[o..o + 4].try_into().unwrap())
            };
            for m in 0..num_models as usize {
                let visible = u32_at(m * 4);
                let ib = counters_bytes as usize + m * 20;
                log::debug!(
                    "model {m}: visible={visible}  args.consumed=[{}, {}, {}, {}, {}]  args.storage=[{}, {}, {}, {}, {}]",
                    u32_at(ib), u32_at(ib + 4), u32_at(ib + 8), u32_at(ib + 12), u32_at(ib + 16),
                    u32_at(args_probe_off as usize + m * 20), u32_at(args_probe_off as usize + m * 20 + 4),
                    u32_at(args_probe_off as usize + m * 20 + 8), u32_at(args_probe_off as usize + m * 20 + 12),
                    u32_at(args_probe_off as usize + m * 20 + 16),
                );
            }
            let f32_at = |o: usize| -> f32 {
                f32::from_le_bytes(data[o..o + 4].try_into().unwrap())
            };
            let dump_matrix = |label: &str, off: usize| {
                let mut mx = [0f32; 16];
                for (k, v) in mx.iter_mut().enumerate() {
                    *v = f32_at(off + k * 4);
                }
                log::debug!("{label} = {mx:?}");
            };
            dump_matrix("mat.vertex_buffer (draw consumes)", args_probe_off as usize);
            dump_matrix("mat.compute_direct (copy source)", mat_probe_off as usize);
            staging.unmap();
        } else {
            log::warn!("ORBITAL_CULL_DEBUG: staging buffer not ready after poll");
        }
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
