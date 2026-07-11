//! Camera realization system — syncs ECS camera components to GPU state.
//!
//! Uses direct `&mut World` access (not IntoSystem) because it needs to
//! query multiple component types and attach new components dynamically.

use std::sync::{Arc, RwLock};

use log::warn;
use orbital_ecs::World;
use orbital_ecs_bridge::{
    CameraDescriptorEcs, CameraDirty, CameraRealization, DeviceResource, Position,
    QueueResource, Rotation,
};
use orbital_resources::Camera;

/// Realize (create or update) GPU camera state for all dirty camera entities.
///
/// Call this each frame before building the bind group. It:
/// 1. Iterates entities with CameraDescriptorEcs + Position + Rotation
/// 2. Creates/updates `CameraRealization` for dirty or newly-added cameras
/// 3. Clears dirty flags
pub fn realize_cameras(ecs: &mut World) {
    // Collect device/queue Arcs
    let (device, queue) = {
        let d = match ecs.get_resource::<DeviceResource>() {
            Some(d) => d.0.clone(),
            None => return, // No GPU device yet
        };
        let q = match ecs.get_resource::<QueueResource>() {
            Some(q) => q.0.clone(),
            None => return,
        };
        (d, q)
    };

    // Collect entities that need realization
    let entities_to_realize: Vec<(usize, CameraDescriptorEcs, Position, Rotation, bool)> = {
        let descs = ecs.get_component_store::<CameraDescriptorEcs>();
        let positions = ecs.get_component_store::<Position>();
        let rotations = ecs.get_component_store::<Rotation>();

        let (descs, positions, rotations) = match (descs, positions, rotations) {
            (Some(d), Some(p), Some(r)) => (d, p, r),
            _ => return,
        };

        let mut result = Vec::new();
        for &eid in descs.dense.as_slice() {
            let desc_idx = match descs.sparse[eid] {
                Some(i) => i,
                None => continue,
            };
            let pos_idx = match positions.sparse[eid] {
                Some(i) => i,
                None => continue,
            };
            let rot_idx = match rotations.sparse[eid] {
                Some(i) => i,
                None => continue,
            };

            let is_dirty = match ecs.get_component_store::<CameraDirty>() {
                Some(store) => match store.sparse[eid] {
                    Some(idx) => store.components[idx].0,
                    None => true,
                },
                None => true, // No dirty flag = needs initial realization
            };

            let has_realization = ecs
                .get_component_store::<CameraRealization>()
                .map(|store| store.sparse[eid].is_some())
                .unwrap_or(false);

            if !is_dirty && has_realization {
                continue;
            }

            result.push((
                eid,
                descs.components[desc_idx].clone(),
                positions.components[pos_idx],
                rotations.components[rot_idx],
                !has_realization, // true = needs new realization
            ));
        }
        result
    };

    // Realize each camera
    for (eid, desc, pos, rot, needs_new) in entities_to_realize {
        if needs_new {
            // Create new GPU camera
            let gpu_camera = Camera::new(
                pos.0,
                rot.0,
                desc.fovy.0,
                desc.aspect,
                desc.near,
                desc.far,
                desc.global_gamma,
                &device,
                &queue,
            );
            let entity = orbital_ecs::Entity::new(eid, 0);
            if let Err(e) = ecs.attach_component(
                &entity,
                CameraRealization(Arc::new(RwLock::new(gpu_camera))),
            ) {
                warn!("Failed to attach CameraRealization to entity {}: {:?}", eid, e);
            }
        } else {
            // Update existing GPU camera
            let real_store = match ecs.get_component_store::<CameraRealization>() {
                Some(s) => s,
                None => continue,
            };
            let idx = match real_store.sparse[eid] {
                Some(i) => i,
                None => continue,
            };
            let realization = &real_store.components[idx];
            let mut gpu_camera = realization.0.write().unwrap();
            gpu_camera.update_from_parts(
                pos.0,
                rot.0,
                desc.fovy.0,
                desc.aspect,
                desc.near,
                desc.far,
                desc.global_gamma,
                &queue,
            );
        }

        // Clear dirty flag
        if let Some(dirty_store) = ecs.get_component_store_mut::<CameraDirty>() {
            if let Some(idx) = dirty_store.sparse[eid] {
                dirty_store.get_mut_store().components[idx].0 = false;
            }
        }
    }
}
