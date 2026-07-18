use cgmath::{Point3, Vector4};
use orbital_ecs::World;
use orbital_ecs_bridge::{
    ActiveCamera, CullResource, DeviceResource, EcsCameraStore, FrozenFrustum, ModelInstances,
    ModelRealization, QueueResource,
};
use orbital_resources::{CullModelInfo, Instance, Transform};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

/// Per‑frame frustum‑culling system.
///
/// Call **after** `realize_models()` (so all GPU models exist) and **before**
/// extraction. Writes [`CullResource`] which the renderer uses to skip
/// non‑visible instances.
pub fn sys_frustum_cull(ecs: &mut World) {
    // --- Device / queue ----------------------------------------------------
    let (device, _queue) = {
        let d = match ecs.get_resource::<DeviceResource>() {
            Some(d) => d.0.clone(),
            None => return,
        };
        let q = match ecs.get_resource::<QueueResource>() {
            Some(q) => q.0.clone(),
            None => return,
        };
        (d, q) // queue kept around in case future culling needs write_buffer
    };

    // --- Camera frustum ----------------------------------------------------
    // Use frozen frustum when set, otherwise derive from the live camera.
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

    // --- Model stores ------------------------------------------------------
    // Scope all immutable borrows so the final insert_resource works.
    let cull_info: Vec<CullModelInfo> = {
        let realizations = match ecs.get_component_store::<ModelRealization>() {
            Some(s) => s,
            None => return,
        };
        let instances_store = match ecs.get_component_store::<ModelInstances>() {
            Some(s) => s,
            None => return,
        };

        let mut info: Vec<CullModelInfo> = Vec::new();

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
            let total_count = model.instance_count();

            let Some(bsphere) = mesh.bounding_sphere() else {
                // No bounds — assume all visible.
                info.push(CullModelInfo {
                    visible_count: total_count,
                    visible_buffer: model.instance_buffer().clone(),
                });
                continue;
            };

            // Test each instance against the frustum.
            let visible_transforms: Vec<&Transform> = model_instances
                .0
                .values()
                .filter(|transform| {
                    let m = transform.to_matrix();
                    let h =
                        m * Vector4::new(bsphere.center.x, bsphere.center.y, bsphere.center.z, 1.0);
                    let world_center = Point3::new(h.x, h.y, h.z);

                    let max_scale = transform
                        .scale
                        .x
                        .max(transform.scale.y)
                        .max(transform.scale.z);
                    let world_radius = bsphere.radius * max_scale;

                    frustum.intersects_sphere(&world_center, world_radius)
                })
                .collect();

            let visible_count = visible_transforms.len() as u32;

            if visible_count == total_count {
                // All visible — reuse the original instance buffer.
                info.push(CullModelInfo {
                    visible_count,
                    visible_buffer: model.instance_buffer().clone(),
                });
            } else if visible_count == 0 {
                // Nothing visible.
                info.push(CullModelInfo::empty(&device));
            } else {
                // Build a filtered buffer with only the visible instances.
                let instances: Vec<Instance> = visible_transforms
                    .iter()
                    .map(|t| Instance::from(*t))
                    .collect();
                let buffer_data: Vec<u8> = instances
                    .iter()
                    .flat_map(|i| i.to_buffer_data_flattened())
                    .collect();
                let buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Culled Instance Buffer"),
                    contents: &buffer_data,
                    usage: wgpu::BufferUsages::VERTEX,
                });
                info.push(CullModelInfo {
                    visible_count,
                    visible_buffer: buffer,
                });
            }
        }
        info
    };

    ecs.insert_resource(CullResource(cull_info));
}
