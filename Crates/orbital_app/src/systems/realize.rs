//! Camera and model realization systems — sync ECS components to GPU state.
//!
//! Uses direct `&mut World` access (not IntoSystem) because these functions
//! need to query multiple component types and attach new components dynamically.

use std::sync::{Arc, RwLock};

use log::warn;
use orbital_ecs::World;
use orbital_ecs_bridge::{
    CameraDescriptorEcs, CameraDirty, CameraRealization, DeviceResource, EcsCameraStore,
    EnvironmentDescriptorResource, EnvironmentGpuResource, LightBufferResource, LightDescriptorEcs,
    LightDirty, LightSlotIndex, LightSlotTracker, MAX_LIGHTS, MaterialCacheResource,
    MeshCacheResource, ModelDescriptorEcs, ModelDirty, ModelInstances, ModelRealization, Position,
    PrevPosition, QueueResource, Rotation, ShadowDirtyFlag, SurfaceFormatResource,
};
use orbital_resources::{Camera, Model, WorldEnvironmentDescriptor};

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

    // Realize each camera — write to EcsCameraStore and mark entity
    for (eid, desc, pos, rot, needs_new) in entities_to_realize {
        if needs_new {
            // Create new GPU camera
            let gpu_camera = Camera::new(
                pos.0,
                rot.0,
                desc.fovy.0.to_degrees(),
                desc.aspect,
                desc.near,
                desc.far,
                desc.global_gamma,
                &device,
                &queue,
            );
            let arc_camera = Arc::new(RwLock::new(gpu_camera));

            // Store in EcsCameraStore
            if let Some(mut store) = ecs.get_resource_mut::<EcsCameraStore>() {
                store.insert(eid, arc_camera);
            }

            // Mark entity as realized
            let generation = ecs.generation(eid);
            let entity = orbital_ecs::Entity::new(eid, generation);
            if let Err(e) = ecs.attach_component(&entity, CameraRealization) {
                warn!(
                    "Failed to attach CameraRealization marker to entity {}: {:?}",
                    eid, e
                );
            }
        } else {
            // Update existing GPU camera via EcsCameraStore
            if let Some(store) = ecs.get_resource::<EcsCameraStore>()
                && let Some(arc_camera) = store.get(eid)
            {
                let mut gpu_camera = arc_camera.write().unwrap();
                gpu_camera.update_from_parts(
                    pos.0,
                    rot.0,
                    desc.fovy.0.to_degrees(),
                    desc.aspect,
                    desc.near,
                    desc.far,
                    desc.global_gamma,
                    &queue,
                );
            }
        }

        // Clear dirty flag
        if let Some(dirty_store) = ecs.get_component_store_mut::<CameraDirty>()
            && let Some(idx) = dirty_store.sparse[eid]
        {
            dirty_store.get_mut_store().components[idx].0 = false;
        }
    }
}

/// Realize (create or update) GPU model state for all dirty model entities.
///
/// Call this each frame before rendering. It:
/// 1. Iterates entities with ModelDescriptorEcs + ModelInstances + ModelDirty
/// 2. Creates/updates `ModelRealization` for dirty or newly-added models
/// 3. Clears dirty flags
pub fn realize_models(ecs: &mut World) {
    // Collect device/queue/surface_format
    let (device, queue, surface_format) = {
        let d = match ecs.get_resource::<DeviceResource>() {
            Some(d) => d.0.clone(),
            None => return,
        };
        let q = match ecs.get_resource::<QueueResource>() {
            Some(q) => q.0.clone(),
            None => return,
        };
        let sf = match ecs.get_resource::<SurfaceFormatResource>() {
            Some(f) => f.0,
            None => return,
        };
        (d, q, sf)
    };

    // Get cache references (Arc-wrapped RwLocks, cheap to clone)
    let (mesh_cache, material_cache) = {
        let mc = match ecs.get_resource::<MeshCacheResource>() {
            Some(c) => Arc::clone(&c),
            None => return,
        };
        let mtc = match ecs.get_resource::<MaterialCacheResource>() {
            Some(c) => Arc::clone(&c),
            None => return,
        };
        (mc, mtc)
    };

    // Collect entities that need realization
    let entities_to_realize: Vec<(usize, ModelDescriptorEcs, ModelInstances, bool)> = {
        let descs = match ecs.get_component_store::<ModelDescriptorEcs>() {
            Some(s) => s,
            None => return,
        };
        let instances = match ecs.get_component_store::<ModelInstances>() {
            Some(s) => s,
            None => return,
        };

        let mut result = Vec::new();
        for &eid in descs.dense.as_slice() {
            let desc_idx = match descs.sparse[eid] {
                Some(i) => i,
                None => continue,
            };
            let inst_idx = match instances.sparse[eid] {
                Some(i) => i,
                None => continue,
            };

            let is_dirty = match ecs.get_component_store::<ModelDirty>() {
                Some(store) => match store.sparse.get(eid).and_then(|x| *x) {
                    Some(idx) => store.components[idx].0,
                    None => true,
                },
                None => true,
            };

            let has_realization = ecs
                .get_component_store::<ModelRealization>()
                .map(|store| store.sparse.get(eid).is_some())
                .unwrap_or(false);

            if !is_dirty && has_realization {
                continue;
            }

            result.push((
                eid,
                descs.components[desc_idx].clone(),
                instances.components[inst_idx].clone(),
                !has_realization,
            ));
        }
        result
    };

    // Realize each model
    for (eid, desc, instances, needs_new) in entities_to_realize {
        // Build a ModelDescriptor from ECS components
        let model_desc = orbital_resources::ModelDescriptor {
            label: desc.label.clone(),
            mesh: desc.mesh.clone(),
            materials: desc.materials.clone(),
            transforms: instances.0.clone(),
        };

        match Model::from_descriptor(
            &model_desc,
            &surface_format,
            &device,
            &queue,
            &mesh_cache,
            &material_cache,
        ) {
            Ok(gpu_model) => {
                if needs_new {
                    let generation = ecs.generation(eid);
                    let entity = orbital_ecs::Entity::new(eid, generation);
                    if let Err(e) =
                        ecs.attach_component(&entity, ModelRealization(Arc::new(gpu_model)))
                    {
                        warn!(
                            "Failed to attach ModelRealization to entity {}: {:?}",
                            eid, e
                        );
                    }
                } else {
                    // Update existing realization by replacing the Arc
                    if let Some(store) = ecs.get_component_store_mut::<ModelRealization>()
                        && let Some(Some(idx)) = store.sparse.get(eid)
                    {
                        store.get_mut_store().components[*idx] =
                            ModelRealization(Arc::new(gpu_model));
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Failed to realize model '{}' (entity {}): {:?}",
                    desc.label, eid, e
                );
            }
        }

        // Clear dirty flag
        if let Some(dirty_store) = ecs.get_component_store_mut::<ModelDirty>()
            && let Some(idx) = dirty_store.sparse[eid]
        {
            dirty_store.get_mut_store().components[idx].0 = false;
        }
    }
}

/// Realize the unified light GPU buffer.
///
/// On first call, pre-allocates a fixed-capacity storage buffer
/// (MAX_LIGHTS × 64 bytes). Each light gets a stable slot index.
/// Dirty lights are written incrementally — only their 64-byte slot
/// is uploaded, not the entire buffer. Lights whose positions have
/// changed are flagged with `ShadowDirtyFlag` for shadow invalidation.
pub fn realize_lights(ecs: &mut World) {
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

    let max_slots = MAX_LIGHTS as u64;

    // Ensure the pre-allocated buffer exists (created once on first frame)
    {
        let existing = ecs.get_resource::<LightBufferResource>();
        let needs_create = existing.is_none_or(|r| r.0.is_none());
        if needs_create {
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ECS Light Buffer (pre-allocated)"),
                size: max_slots * 64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            ecs.insert_resource(LightBufferResource(Some(Arc::new(buf))));
        }
    }

    // Ensure the slot tracker exists
    if ecs.get_resource::<LightSlotTracker>().is_none() {
        ecs.insert_resource(LightSlotTracker::new());
    }

    // Fast preliminary check — is there any work at all?
    // Scans for dirty lights, new lights (lacking LightSlotIndex), moved lights,
    // and removed entities without building full LightDescriptor objects.
    let (has_work, needs_full_pass) = {
        let descs_store = match ecs.get_component_store::<LightDescriptorEcs>() {
            Some(s) => s,
            None => return,
        };
        let slot_store = ecs.get_component_store::<LightSlotIndex>();
        let dirty_store = ecs.get_component_store::<LightDirty>();
        let prev_pos_store = ecs.get_component_store::<PrevPosition>();
        let positions_store = ecs.get_component_store::<Position>();

        let mut found_dirty = false;
        let mut found_new = false;
        let mut found_moved = false;

        for &eid in descs_store.dense.as_slice() {
            // Check if this entity actually has a descriptor
            if descs_store.sparse.get(eid).copied().flatten().is_none() {
                continue;
            }

            // Check dirty flag
            if !found_dirty
                && let Some(ref ds) = dirty_store
                && let Some(d) = ds.get_component(eid)
                && d.is_dirty()
            {
                found_dirty = true;
            }

            // Check if new (no LightSlotIndex yet)
            if !found_new {
                if let Some(ref ss) = slot_store {
                    if ss.get_component(eid).is_none() {
                        found_new = true;
                    }
                } else {
                    // No slot store at all → every light is new
                    found_new = true;
                }
            }

            // Check position changes
            if !found_moved
                && let Some(ref ps) = prev_pos_store
                && let Some(pp) = ps.get_component(eid)
                && let Some(ref pos) = positions_store
                && let Some(ppi) = pos.get_component(eid)
            {
                let diff =
                    (ppi.0.x - pp.0.x).abs() + (ppi.0.y - pp.0.y).abs() + (ppi.0.z - pp.0.z).abs();
                if diff > 1e-6 {
                    found_moved = true;
                }
            }

            if found_dirty && found_new && found_moved {
                break;
            }
        }

        // Check for removed entities (in tracker but not in descriptor store)
        let has_removals = if let Some(tracker) = ecs.get_resource::<LightSlotTracker>() {
            let current: std::collections::HashSet<usize> =
                descs_store.dense.iter().copied().collect();
            tracker
                .entity_to_slot
                .iter()
                .enumerate()
                .any(|(eid, slot)| slot.is_some() && !current.contains(&eid))
        } else {
            false
        };

        (
            found_dirty || found_new || found_moved || has_removals,
            found_new,
        )
    };

    if needs_full_pass {
        ecs.insert_resource(orbital_ecs_bridge::NewLightBootstrap(true));
    }

    if !has_work {
        return;
    }

    // --- Phase 1: Collect full data ---

    let descs_store = match ecs.get_component_store::<LightDescriptorEcs>() {
        Some(s) => s,
        None => return,
    };
    let positions_store = match ecs.get_component_store::<Position>() {
        Some(s) => s,
        None => return,
    };

    let light_entities: Vec<usize> = descs_store.dense.clone();

    struct LightSlotInfo {
        eid: usize,
        slot_idx: u32,
        is_new: bool,
        is_dirty: bool,
        position_moved: bool,
        pos: cgmath::Point3<f32>,
        desc: orbital_resources::LightDescriptor,
    }

    let mut slot_infos: Vec<LightSlotInfo> = Vec::new();
    let slot_idx_store = ecs.get_component_store::<LightSlotIndex>();
    let dirty_store = ecs.get_component_store::<LightDirty>();
    let prev_pos_store = ecs.get_component_store::<PrevPosition>();

    for &eid in &light_entities {
        let desc_idx = match descs_store.sparse.get(eid).copied().flatten() {
            Some(i) => i,
            None => continue,
        };
        let desc_component = &descs_store.components[desc_idx];

        let pos = positions_store
            .sparse
            .get(eid)
            .copied()
            .flatten()
            .map(|pi| positions_store.components[pi].0)
            .unwrap_or(cgmath::Point3::new(0.0, 0.0, 0.0));

        let is_dirty = dirty_store
            .as_ref()
            .and_then(|s| s.get_component(eid))
            .map(|d| d.is_dirty())
            .unwrap_or(false);

        let existing_slot = slot_idx_store
            .as_ref()
            .and_then(|s| s.get_component(eid))
            .map(|si| si.0);

        let prev_pos = prev_pos_store
            .as_ref()
            .and_then(|s| s.get_component(eid))
            .map(|pp| pp.0);

        let position_moved = match prev_pos {
            Some(pp) => {
                (pos.x - pp.x).abs() > 1e-6
                    || (pos.y - pp.y).abs() > 1e-6
                    || (pos.z - pp.z).abs() > 1e-6
            }
            None => false,
        };

        let slot_idx = if let Some(si) = existing_slot {
            si
        } else {
            let tracker = ecs.get_resource_mut::<LightSlotTracker>();
            match tracker {
                Some(mut t) => t.allocate(eid),
                None => 0,
            }
        };

        let light_desc = orbital_resources::LightDescriptor {
            label: String::new(),
            light_type: desc_component.light_type.clone(),
            color: desc_component.color,
            position: cgmath::Vector3::new(pos.x, pos.y, pos.z),
            direction: desc_component.direction,
        };

        slot_infos.push(LightSlotInfo {
            eid,
            slot_idx,
            is_new: existing_slot.is_none(),
            is_dirty,
            position_moved,
            pos,
            desc: light_desc,
        });
    }

    // Drop all read handles before mutation phase
    drop(slot_idx_store);
    drop(dirty_store);
    drop(prev_pos_store);
    drop(positions_store);
    drop(descs_store);

    // --- Phase 2: Mutate state and upload GPU data ---

    let light_buffer = match ecs.get_resource::<LightBufferResource>() {
        Some(r) => match &r.0 {
            Some(buf) => buf.as_ref().clone(),
            None => return,
        },
        None => return,
    };

    // Handle removal: find slots assigned to entities no longer present
    let mut freed_slots: Vec<u32> = Vec::new();
    {
        let current_set: std::collections::HashSet<usize> =
            light_entities.iter().copied().collect();
        let tracker = ecs.get_resource::<LightSlotTracker>();
        if let Some(t) = tracker {
            let zero_buf = vec![0u8; 64];
            for (stale_eid, slot_opt) in t.entity_to_slot.iter().enumerate() {
                if let Some(slot) = slot_opt {
                    let slot = *slot;
                    if !current_set.contains(&stale_eid) && (slot as u64) < max_slots {
                        queue.write_buffer(&light_buffer, slot as u64 * 64, &zero_buf);
                        freed_slots.push(slot);
                    }
                }
            }
        }
    }

    // Free removed slots in the tracker
    if !freed_slots.is_empty()
        && let Some(mut tracker) = ecs.get_resource_mut::<LightSlotTracker>()
    {
        for slot in &freed_slots {
            for entry in tracker.entity_to_slot.iter_mut() {
                if *entry == Some(*slot) {
                    *entry = None;
                }
            }
            tracker.free_slots.push(*slot);
        }
    }

    for info in &slot_infos {
        // Attach LightSlotIndex for new lights
        if info.is_new {
            let light_entity = orbital_ecs::Entity::new(info.eid, ecs.generation(info.eid));
            if ecs
                .attach_component(&light_entity, LightSlotIndex(info.slot_idx))
                .is_err()
            {
                warn!(
                    "realize_lights: failed to attach LightSlotIndex to entity {}",
                    info.eid
                );
            }
            if ecs
                .attach_component(&light_entity, ShadowDirtyFlag(true))
                .is_err()
            {
                warn!(
                    "realize_lights: failed to attach ShadowDirtyFlag to entity {}",
                    info.eid
                );
            }
            if ecs
                .attach_component(&light_entity, PrevPosition(info.pos))
                .is_err()
            {
                warn!(
                    "realize_lights: failed to attach PrevPosition to entity {}",
                    info.eid
                );
            }
        }

        // Upload GPU data for dirty lights
        if info.is_dirty {
            let data = info.desc.to_buffer_data();
            if (info.slot_idx as u64) < max_slots {
                queue.write_buffer(&light_buffer, info.slot_idx as u64 * 64, &data);
            }
        }

        // Mark shadow dirty if position moved
        if info.position_moved {
            if let Some(mut store) = ecs.get_component_store_mut::<ShadowDirtyFlag>()
                && let Some(idx) = store.sparse.get(info.eid).copied().flatten()
            {
                store.components[idx].mark_dirty();
            }
            if let Some(mut store) = ecs.get_component_store_mut::<PrevPosition>()
                && let Some(idx) = store.sparse.get(info.eid).copied().flatten()
            {
                store.components[idx] = PrevPosition(info.pos);
            }
        }

        // Light property dirty → shadow also dirty
        if info.is_dirty
            && let Some(mut store) = ecs.get_component_store_mut::<ShadowDirtyFlag>()
            && let Some(idx) = store.sparse.get(info.eid).copied().flatten()
        {
            store.components[idx].mark_dirty();
        }
    }

    // Clear LightDirty flags
    if let Some(mut store) = ecs.get_component_store_mut::<LightDirty>() {
        for &eid in &light_entities {
            if let Some(idx) = store.sparse.get(eid).copied().flatten() {
                store.components[idx].clear();
            }
        }
    }
}

/// Realize the world environment (IBL textures, skybox) from the descriptor.
///
/// If no descriptor has ever been set, automatically generates a default
/// procedural atmospheric-scattering sky.  Use `WorldEnvironmentDescriptor::None`
/// to explicitly opt out of IBL / skybox.
pub fn realize_environment(ecs: &mut World) {
    let (device, queue, surface_format) = {
        let d = match ecs.get_resource::<DeviceResource>() {
            Some(d) => d.0.clone(),
            None => return,
        };
        let q = match ecs.get_resource::<QueueResource>() {
            Some(q) => q.0.clone(),
            None => return,
        };
        let sf = match ecs.get_resource::<SurfaceFormatResource>() {
            Some(f) => f.0,
            None => return,
        };
        (d, q, sf)
    };

    // If the environment is already realized AND no new descriptor is
    // waiting, there is nothing to do.
    let gpu_exists = ecs
        .get_resource::<EnvironmentGpuResource>()
        .map_or(false, |r| r.0.is_some());

    let descriptor_opt = ecs
        .get_resource::<EnvironmentDescriptorResource>()
        .and_then(|r| r.0.clone());

    if gpu_exists && descriptor_opt.is_none() {
        return;
    }

    // Determine which descriptor to use.
    let descriptor = match descriptor_opt {
        Some(WorldEnvironmentDescriptor::None) => {
            // Explicitly disable the environment (no skybox, no IBL).
            ecs.insert_resource(EnvironmentGpuResource(None));
            ecs.insert_resource(EnvironmentDescriptorResource(None));
            return;
        }
        Some(d) => d,
        None => {
            // No descriptor was ever set — use a default procedural sky.
            WorldEnvironmentDescriptor::Generated {
                cube_face_size: WorldEnvironmentDescriptor::DEFAULT_SIZE,
                sampling_type: WorldEnvironmentDescriptor::DEFAULT_SAMPLING_TYPE,
                custom_specular_mip_level_count: None,
                parameters: None,
            }
        }
    };

    match orbital_resources::WorldEnvironment::from_descriptor(
        &descriptor,
        Some(surface_format),
        &device,
        &queue,
    ) {
        Ok(env) => {
            ecs.insert_resource(EnvironmentGpuResource(Some(Arc::new(env))));
            // Clear the descriptor (consumed)
            ecs.insert_resource(EnvironmentDescriptorResource(None));
        }
        Err(e) => {
            warn!("Failed to realize environment: {:?}", e);
        }
    }
}
