use cgmath::Vector4;
use orbital_ecs::World;
use orbital_ecs_bridge::{
    ActiveCamera, CullResource, DeviceResource, EcsCameraStore, FrozenFrustum, ModelInstances,
    ModelRealization, QueueResource,
};
use orbital_resources::{CullResources, Instance};
use wgpu::util::DeviceExt;

/// Per‑frame GPU‑accelerated frustum‑culling system.
///
/// Call **after** `realize_models()` and **before** extraction.
pub fn sys_frustum_cull(ecs: &mut World) {
    // Log the resolved cull debug flags + storage root once per process so
    // on-device tests are self-verifying from logcat (marker files are
    // platform + launch-timing sensitive on Android).
    orbital_core::debug_flags::log_active_flags();

    // ── Device / queue ────────────────────────────────────────────────
    let (device, queue) = {
        let d = match ecs.get_resource::<DeviceResource>() {
            Some(d) => d.0.clone(),
            None => return,
        };
        let q = match ecs.get_resource::<QueueResource>() {
            Some(q) => q.0.clone(),
            None => return,
        };
        (d, q)
    };

    // ── Frustum (read‑only borrows, scoped) ───────────────────────────
    let frustum = {
        let frozen = ecs
            .get_resource::<FrozenFrustum>()
            .and_then(|f| f.0.clone());
        if let Some(ref data) = frozen {
            data.frustum.clone()
        } else {
            let active = match ecs.get_resource::<ActiveCamera>() {
                Some(a) => a.0,
                None => return,
            };
            let store = match ecs.get_resource::<EcsCameraStore>() {
                Some(s) => s,
                None => return,
            };
            let arc_cam = match store.get(active.index) {
                Some(c) => c,
                None => return,
            };
            let cam = arc_cam.read().unwrap();
            cam.frustum()
        }
    };

    // ── Build entries (all immutable borrows scoped here) ─────────────
    #[derive(Default)]
    struct Entry {
        first_instance: u32,
        instance_count: u32,
        index_count: u32,
        first_index: u32,
        base_vertex: i32,
        instance_bytes: Vec<u8>,
        bounds_bytes: Vec<u8>,
    }

    let entries: Vec<Entry> = {
        let realizations = match ecs.get_component_store::<ModelRealization>() {
            Some(s) => s,
            None => return,
        };
        let instances_store = match ecs.get_component_store::<ModelInstances>() {
            Some(s) => s,
            None => return,
        };

        let mut entries: Vec<Entry> = Vec::new();
        let mut total: u32 = 0;

        for &eid in realizations.dense.as_slice() {
            let Some(real_idx) = realizations.sparse[eid] else {
                continue;
            };
            let Some(inst_idx) = instances_store.sparse[eid] else {
                continue;
            };

            let model = &realizations.components[real_idx].0;
            let model_instances = &instances_store.components[inst_idx];
            let mesh = model.mesh();
            let count = model.instance_count();
            let Some(bsphere) = mesh.bounding_sphere() else {
                continue;
            };

            let mut inst_bytes = Vec::with_capacity(count as usize * 64);
            let mut bounds_bytes = Vec::with_capacity(count as usize * 16);

            for transform in model_instances.0.values() {
                let instance = Instance::from(transform);
                inst_bytes.extend(instance.to_buffer_data_flattened());

                let m = transform.to_matrix();
                let h = m * Vector4::new(bsphere.center.x, bsphere.center.y, bsphere.center.z, 1.0);
                let max_scale = transform
                    .scale
                    .x
                    .max(transform.scale.y)
                    .max(transform.scale.z);
                bounds_bytes.extend_from_slice(&h.x.to_le_bytes());
                bounds_bytes.extend_from_slice(&h.y.to_le_bytes());
                bounds_bytes.extend_from_slice(&h.z.to_le_bytes());
                bounds_bytes.extend_from_slice(&(bsphere.radius * max_scale).to_le_bytes());
            }

            entries.push(Entry {
                first_instance: total,
                instance_count: count,
                index_count: mesh.index_count(),
                first_index: 0,
                base_vertex: 0,
                instance_bytes: inst_bytes,
                bounds_bytes,
            });
            total += count;
        }
        entries
    };

    if entries.is_empty() {
        ecs.insert_resource(CullResource(None));
        return;
    }

    let num_models = entries.len() as u32;
    let total_instances: u32 = entries
        .last()
        .map(|e| e.first_instance + e.instance_count)
        .unwrap_or(0);
    let max_inst_per_model = entries.iter().map(|e| e.instance_count).max().unwrap_or(1);

    // ── Ensure CullResources exists with sufficient capacity ──────────
    let existing_info = ecs
        .get_resource::<CullResource>()
        .map(|r| r.0.as_ref().map(|cr| (cr.max_instances(), cr.max_models())));
    // `cull_all` (the `orbital_cull_all` marker / `ORBITAL_CULL_ALL=1`, or the
    // legacy `ORBITAL_CULL_DEBUG=cull_all`) needs that entry point compiled
    // into the pipelines — force a (re)allocation so the resource is always
    // built with it enabled.
    let debug_cull_all = orbital_core::debug_flags::cull_all()
        || orbital_core::debug_flags::cull_debug_mode()
            == orbital_core::debug_flags::CullDebugMode::CullAll;
    // Single-encoder mode: cull compute is submitted with the render pass
    // (inside the renderer's encoder) rather than in its own submission.
    // Forces a fresh allocation so the flag is carried on the resource.
    let debug_single_encoder = orbital_core::debug_flags::cull_single_encoder();
    let needs_alloc = if debug_cull_all || debug_single_encoder {
        true
    } else {
        match existing_info {
            Some(Some((max_inst, max_mdl))) => {
                max_inst < total_instances || max_mdl < num_models
            }
            _ => true,
        }
    };
    if needs_alloc {
        ecs.insert_resource(CullResource(Some(CullResources::with_debug(
            &device,
            total_instances,
            num_models,
            debug_cull_all,
            debug_single_encoder,
        ))));
    }

    // ── Get mutable access & upload ───────────────────────────────────
    let Some(mut guard) = ecs.get_resource_mut::<CullResource>() else {
        return;
    };
    let Some(ref mut cr) = guard.0 else { return };

    cr.upload_frustum(&queue, &frustum);

    // Per-model params + offsets
    // Per-model params are packed as 8 × u32 = 32 bytes per model to match
    // the shader, which reads each model as two `vec4<u32>` (`params[model*2]`
    // and `params[model*2+1]`). The host layout is:
    //   [0] = (first_instance, instance_count, index_count, first_index)
    //   [1] = (base_vertex as u32, pad, pad, pad)
    let mut params_bytes = Vec::with_capacity(entries.len() * 32);
    let mut offsets = Vec::with_capacity(entries.len());
    for e in &entries {
        params_bytes.extend_from_slice(&e.first_instance.to_le_bytes());
        params_bytes.extend_from_slice(&e.instance_count.to_le_bytes());
        params_bytes.extend_from_slice(&e.index_count.to_le_bytes());
        params_bytes.extend_from_slice(&e.first_index.to_le_bytes());
        params_bytes.extend_from_slice(&(e.base_vertex as u32).to_le_bytes());
        params_bytes.extend_from_slice(&0u32.to_le_bytes()); // pad
        params_bytes.extend_from_slice(&0u32.to_le_bytes()); // pad (already part of
        params_bytes.extend_from_slice(&0u32.to_le_bytes()); // the empty vec4[1])
        offsets.push(e.first_instance);
    }
    cr.set_model_offsets(offsets);
    cr.upload_params(&queue, &params_bytes);

    // Instances + bounds
    let all_inst: Vec<u8> = entries
        .iter()
        .flat_map(|e| e.instance_bytes.clone())
        .collect();
    let all_bounds: Vec<u8> = entries
        .iter()
        .flat_map(|e| e.bounds_bytes.clone())
        .collect();
    cr.upload_instances_and_bounds(&queue, &all_inst, &all_bounds);

    // Drop guard so we can borrow ecs again for encoder creation.
    drop(guard);

    // In single-encoder mode the cull compute is dispatched inside the
    // renderer's submission (`CullResources::dispatch_into_render`), so we
    // skip the separate zero-counters and compute submissions here. Counters
    // are still self-resetting: `finalize` atomicExchange's them back to 0
    // every frame, and the buffers are created zero-initialised.
    if debug_single_encoder {
        return;
    }

    // ── Zero counters via buffer copy ─────────────────────────────────
    {
        let zero_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cull Zero Init"),
            contents: &vec![0u8; num_models as usize * 4],
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        let mut cmd_enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Cull Init Encoder"),
        });
        let guard2 = ecs.get_resource::<CullResource>();
        let cr2 = match guard2 {
            Some(ref r) => match r.0 {
                Some(ref c) => c,
                None => return,
            },
            None => return,
        };
        cmd_enc.copy_buffer_to_buffer(
            &zero_buf,
            0,
            cr2.counters_buffer(),
            0,
            num_models as u64 * 4,
        );
        queue.submit(vec![cmd_enc.finish()]);
    }

    // ── Dispatch compute ──────────────────────────────────────────────
    {
        let mut cmd_enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Cull Compute Encoder"),
        });
        let guard3 = ecs.get_resource::<CullResource>();
        let cr3 = match guard3 {
            Some(ref r) => match r.0 {
                Some(ref c) => c,
                None => return,
            },
            None => return,
        };
        cr3.dispatch(&mut cmd_enc, num_models, max_inst_per_model, debug_cull_all);
        queue.submit(vec![cmd_enc.finish()]);
    }
}
