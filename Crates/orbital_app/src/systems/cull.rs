use cgmath::{Point3, Vector4};
use orbital_ecs::World;
use orbital_ecs_bridge::{
    ActiveCamera, CullResource, DeviceResource, EcsCameraStore, FrozenFrustum, ModelInstances,
    ModelRealization, QueueResource,
};
use orbital_resources::{CullResources, Instance};

/// Per-frame CPU frustum-culling system.
///
/// Tests each instance's bounding sphere against the camera frustum on the
/// CPU.  Visible instance matrices are compacted into a vertex buffer that
/// the renderer consumes via direct `draw_indexed` calls.
///
/// Call **after** `realize_models()` and **before** extraction.
pub fn sys_frustum_cull(ecs: &mut World) {
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
        instance_count: u32,
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
                instance_count: count,
                instance_bytes: inst_bytes,
                bounds_bytes,
            });
        }
        entries
    };

    if entries.is_empty() {
        ecs.insert_resource(CullResource(None));
        return;
    }

    let num_models = entries.len() as u32;
    let total_instances: u32 = entries.iter().map(|e| e.instance_count).sum();

    // ── Ensure CullResources exists with sufficient capacity ──────────
    // Always recreate to ensure buffer is large enough.
    // A more optimal approach would cache and only reallocate when capacity is exceeded.
    ecs.insert_resource(CullResource(Some(CullResources::new(
        &device,
        total_instances,
        num_models,
    ))));

    // ── CPU frustum cull + upload ─────────────────────────────────────
    let Some(mut guard) = ecs.get_resource_mut::<CullResource>() else {
        return;
    };
    let Some(ref mut cr) = guard.0 else {
        return;
    };

    let mut all_visible = Vec::new();
    let mut offsets = Vec::new();
    let mut counts = Vec::new();
    let mut offset = 0u32;

    for entry in &entries {
        offsets.push(offset);
        let mut model_visible = 0u32;

        let instance_count = entry.instance_count as usize;
        for inst_idx in 0..instance_count {
            // Extract pre-computed world-space bounds: [center_x, center_y, center_z, radius]
            let base = inst_idx * 16;
            if base + 16 > entry.bounds_bytes.len() {
                continue;
            }
            let cx = f32::from_le_bytes(entry.bounds_bytes[base..base + 4].try_into().unwrap());
            let cy =
                f32::from_le_bytes(entry.bounds_bytes[base + 4..base + 8].try_into().unwrap());
            let cz =
                f32::from_le_bytes(entry.bounds_bytes[base + 8..base + 12].try_into().unwrap());
            let radius =
                f32::from_le_bytes(entry.bounds_bytes[base + 12..base + 16].try_into().unwrap());

            let center = Point3::new(cx, cy, cz);
            if frustum.intersects_sphere(&center, radius) {
                // Copy this instance's 64-byte matrix to the visible list
                let mat_base = inst_idx * 64;
                let mat_end = mat_base + 64;
                if mat_end <= entry.instance_bytes.len() {
                    all_visible
                        .extend_from_slice(&entry.instance_bytes[mat_base..mat_end]);
                    model_visible += 1;
                }
            }
        }

        offset += model_visible;
        counts.push(model_visible);
    }

    let total_visible: u32 = counts.iter().sum();
    log::debug!(
        "CPU frustum cull: {total_visible}/{} instances visible across {num_models} models",
        total_instances,
    );

    cr.write_visible_instances(&queue, &all_visible, offsets, counts);
}
