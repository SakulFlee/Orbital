use std::sync::Arc;

use cgmath::Point3;
use orbital_cull::CullResources;
use orbital_ecs::World;
use orbital_ecs_bridge::{
    ActiveCamera, CameraStore, CullResource, DeviceResource, FrozenFrustum, ModelRealization,
    QueueResource,
};

/// Per-frame CPU frustum-culling system.
///
/// Tests each instance's cached bounding sphere against the camera frustum and
/// compacts visible instance matrices into the persistent cull buffer. Models
/// already own their instance data and bounds, so this no longer re-encodes
/// transforms or allocates per-model byte vectors every frame.
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
            let store = match ecs.get_resource::<CameraStore>() {
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

    // ── Snapshot realized models in renderer order ────────────────────
    let models: Vec<Arc<orbital_model::Model>> = {
        let Some(realizations) = ecs.get_component_store::<ModelRealization>() else {
            return;
        };

        realizations
            .dense
            .iter()
            .filter_map(|&entity_id| {
                let index = realizations.sparse.get(entity_id).copied().flatten()?;
                Some(Arc::clone(&realizations.components[index].0))
            })
            .collect()
    };

    if models.is_empty() {
        ecs.insert_resource(CullResource(None));
        return;
    }

    let num_models = models.len() as u32;
    let total_instances: u32 = models.iter().map(|model| model.instance_count()).sum();
    let required_capacity = total_instances.max(1);

    // ── Ensure CullResources exists with sufficient capacity ──────────
    let needs_reallocation = ecs.get_resource::<CullResource>().is_none_or(|resource| {
        resource.0.as_ref().is_none_or(|cull| {
            cull.capacity() < required_capacity || cull.model_capacity() < num_models
        })
    });
    if needs_reallocation {
        ecs.insert_resource(CullResource(Some(CullResources::new(
            &device,
            required_capacity.next_power_of_two(),
            num_models,
        ))));
    }

    // ── CPU frustum cull + upload ─────────────────────────────────────
    let Some(mut guard) = ecs.get_resource_mut::<CullResource>() else {
        return;
    };
    let Some(ref mut cull) = guard.0 else {
        return;
    };

    cull.begin_frame();
    let mut offset = 0u32;

    for model in &models {
        let bounds = model.instance_bounds();
        let instance_data = model.instance_data();
        let instance_count = model.instance_count() as usize;
        let mut visible = 0u32;

        for (index, bound) in bounds.iter().take(instance_count).enumerate() {
            let center = Point3::new(bound[0], bound[1], bound[2]);
            if !frustum.intersects_sphere(&center, bound[3]) {
                continue;
            }

            let start = index * 64;
            let end = start + 64;
            if end <= instance_data.len() {
                cull.push_instance(&instance_data[start..end]);
                visible += 1;
            }
        }

        cull.push_model(offset, visible);
        offset += visible;
    }

    cull.upload(&queue);
}
