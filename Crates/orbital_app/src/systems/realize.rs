//! Camera and model realization systems — sync ECS components to GPU state.
//!
//! Uses direct `&mut World` access (not IntoSystem) because these functions
//! need to query multiple component types and attach new components dynamically.

use std::sync::{Arc, RwLock};

use log::warn;
use orbital_ecs::World;
use orbital_ecs_bridge::{
    CameraDescriptorEcs, CameraDirty, CameraRealization, DeviceResource,
    EcsCameraStore, EnvironmentDescriptorResource, EnvironmentGpuResource, LightBufferResource,
    LightDescriptorEcs, LightDirty, MaterialCacheResource, MeshCacheResource, ModelDescriptorEcs,
    ModelDirty, ModelInstances, ModelRealization, Position, QueueResource, Rotation,
    SurfaceFormatResource,
};
use orbital_resources::{Camera, Model};

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
            let gen = ecs.generation(eid);
            let entity = orbital_ecs::Entity::new(eid, gen);
            if let Err(e) = ecs.attach_component(&entity, CameraRealization) {
                warn!("Failed to attach CameraRealization marker to entity {}: {:?}", eid, e);
            }
        } else {
            // Update existing GPU camera via EcsCameraStore
            if let Some(store) = ecs.get_resource::<EcsCameraStore>() {
                if let Some(arc_camera) = store.get(eid) {
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
        }

        // Clear dirty flag
        if let Some(dirty_store) = ecs.get_component_store_mut::<CameraDirty>() {
            if let Some(idx) = dirty_store.sparse[eid] {
                dirty_store.get_mut_store().components[idx].0 = false;
            }
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
                Some(store) => match store.sparse[eid] {
                    Some(idx) => store.components[idx].0,
                    None => true,
                },
                None => true,
            };

            let has_realization = ecs
                .get_component_store::<ModelRealization>()
                .map(|store| store.sparse[eid].is_some())
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
                    let gen = ecs.generation(eid);
                    let entity = orbital_ecs::Entity::new(eid, gen);
                    if let Err(e) = ecs.attach_component(
                        &entity,
                        ModelRealization(Arc::new(gpu_model)),
                    ) {
                        warn!("Failed to attach ModelRealization to entity {}: {:?}", eid, e);
                    }
                } else {
                    // Update existing realization by replacing the Arc
                    if let Some(store) = ecs.get_component_store_mut::<ModelRealization>() {
                        if let Some(Some(idx)) = store.sparse.get(eid) {
                            store.get_mut_store().components[*idx] =
                                ModelRealization(Arc::new(gpu_model));
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to realize model '{}' (entity {}): {:?}", desc.label, eid, e);
            }
        }

        // Clear dirty flag
        if let Some(dirty_store) = ecs.get_component_store_mut::<ModelDirty>() {
            if let Some(idx) = dirty_store.sparse[eid] {
                dirty_store.get_mut_store().components[idx].0 = false;
            }
        }
    }
}

/// Realize (rebuild) the unified light GPU buffer from all dirty light entities.
///
/// All lights are packed into a single storage buffer. When any light changes,
/// the entire buffer is rebuilt from all LightDescriptorEcs + Position components.
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

    // Check if any light is dirty
    let any_dirty = match ecs.get_component_store::<LightDirty>() {
        Some(store) => store.dense.iter().any(|&eid| {
            store.sparse[eid]
                .map(|idx| store.components[idx].0)
                .unwrap_or(false)
        }),
        None => return, // No dirty flags means no lights or no changes
    };

    if !any_dirty {
        return;
    }

    // Collect all light descriptors with their positions
    let mut light_data: Vec<(LightDescriptorEcs, Position)> = Vec::new();
    {
        let descs = match ecs.get_component_store::<LightDescriptorEcs>() {
            Some(s) => s,
            None => return,
        };
        let positions = match ecs.get_component_store::<Position>() {
            Some(s) => s,
            None => return,
        };

        for &eid in descs.dense.as_slice() {
            let desc_idx = match descs.sparse[eid] {
                Some(i) => i,
                None => continue,
            };
            let pos_idx = positions.sparse[eid].unwrap_or(0); // default position if none
            light_data.push((
                descs.components[desc_idx].clone(),
                positions.components[pos_idx],
            ));
        }
    }

    // Build light descriptors for the GPU buffer
    let light_descriptors: Vec<orbital_resources::LightDescriptor> = light_data
        .iter()
        .map(|(desc, pos)| {
            orbital_resources::LightDescriptor {
                label: String::new(),
                light_type: desc.light_type.clone(),
                color: desc.color,
                position: cgmath::Vector3::new(pos.0.x, pos.0.y, pos.0.z),
                direction: desc.direction,
            }
        })
        .collect();

    // Pack into buffer data (64 bytes per light)
    let buffer_data: Vec<u8> = light_descriptors
        .iter()
        .flat_map(|ld| ld.to_buffer_data())
        .collect();

    // Create or update the GPU buffer
    let buffer_size = buffer_data.len().max(4) as u64; // min 4 bytes for empty
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ECS Light Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    if !buffer_data.is_empty() {
        queue.write_buffer(&buffer, 0, &buffer_data);
    }

    // Update the ECS resource
    ecs.insert_resource(LightBufferResource(Some(Arc::new(buffer))));

    // Clear all dirty flags
    if let Some(dirty_store) = ecs.get_component_store_mut::<LightDirty>() {
        for &eid in dirty_store.dense.as_slice() {
            if let Some(idx) = dirty_store.sparse[eid] {
                dirty_store.get_mut_store().components[idx].0 = false;
            }
        }
    }
}

/// Realize the world environment (IBL textures, skybox) from the descriptor.
///
/// Only runs when the environment descriptor has changed (new Some value).
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

    // Check if there's a new descriptor to realize
    let descriptor = match ecs.get_resource::<EnvironmentDescriptorResource>() {
        Some(r) => match &r.0 {
            Some(d) => d.clone(),
            None => return, // No environment set
        },
        None => return,
    };

    // Check if already realized (compare would need hash, so just re-realize if descriptor exists)
    // For simplicity, always re-realize when the resource is present.
    // A dirty flag pattern could be added later for optimization.

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
