//! Import system — polls glTF importer and spawns ECS entities for results.
//!
//! Uses direct `&mut World` access (not IntoSystem) because it needs to
//! spawn entities and attach multiple component types.

use std::sync::Arc;

use cgmath::{InnerSpace, Point3, Quaternion, Rotation3, Vector3};
use log::warn;
use orbital_ecs::World;
use orbital_ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CameraDirty, CameraRealization, DeviceResource,
    EcsCameraStore, ImportQueueResource, ImporterResource, ModelDescriptorEcs, ModelDirty,
    ModelInstances, Position, QueueResource, Rotation,
};
use orbital_resources::Camera;

/// Poll the importer for completed results and spawn ECS entities.
///
/// This system:
/// 1. Drains the ImportQueueResource and submits tasks to the Importer
/// 2. Polls the Importer for completed results (EVERY frame, even when queue is empty)
/// 3. Spawns ECS entities for each imported model and camera
pub fn sys_poll_importer(ecs: &mut World) {
    // Step 1: Drain queue and submit tasks to importer (independent of result polling)
    {
        let has_tasks = ecs
            .get_resource_mut::<ImportQueueResource>()
            .map(|mut q| {
                let tasks: Vec<_> = q.0.drain(..).collect();
                if !tasks.is_empty() {
                    if let Some(mut importer) = ecs.get_resource_mut::<ImporterResource>() {
                        for task in tasks {
                            importer.0.register_task(task);
                        }
                    }
                    true
                } else {
                    false
                }
            })
            .unwrap_or(false);
        let _ = has_tasks;
    }

    // Step 2: ALWAYS poll for completed results (even when queue was empty)
    let results = {
        let mut importer = match ecs.get_resource_mut::<ImporterResource>() {
            Some(i) => i,
            None => return,
        };
        importer.0.update()
    };

    if results.is_empty() {
        return;
    }

    // Get device/queue for camera realization
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

    // Step 3: Spawn entities for each import result
    for result in results {
        // Spawn model entities
        for model_desc in result.models {
            let entity = ecs.spawn_entity();

            let desc = ModelDescriptorEcs {
                label: model_desc.label.clone(),
                mesh: model_desc.mesh.clone(),
                materials: model_desc.materials.clone(),
            };
            if let Err(e) = ecs.attach_component(&entity, desc) {
                warn!("Failed to attach ModelDescriptorEcs: {:?}", e);
                continue;
            }

            let first_position = model_desc.transforms.values().next().map(|t| {
                Position(Point3::new(t.position.x, t.position.y, t.position.z))
            });

            let instances = ModelInstances(model_desc.transforms);
            if let Err(e) = ecs.attach_component(&entity, instances) {
                warn!("Failed to attach ModelInstances: {:?}", e);
            }

            if let Err(e) = ecs.attach_component(&entity, ModelDirty(true)) {
                warn!("Failed to attach ModelDirty: {:?}", e);
            }

            // Set initial position from first transform if available
            if let Some(pos) = first_position {
                if let Err(e) = ecs.attach_component(&entity, pos) {
                    warn!("Failed to attach Position: {:?}", e);
                }
            }
        }

        // Spawn camera entities
        for cam_desc in result.cameras {
            let entity = ecs.spawn_entity();

            let pos = Position(Point3::new(
                cam_desc.position.x,
                cam_desc.position.y,
                cam_desc.position.z,
            ));
            if let Err(e) = ecs.attach_component(&entity, pos) {
                warn!("Failed to attach Position for camera: {:?}", e);
                continue;
            }

            // Convert yaw/pitch/roll to quaternion
            let q_yaw =
                Quaternion::from_axis_angle(Vector3::unit_y(), cgmath::Rad(cam_desc.yaw));
            let q_pitch =
                Quaternion::from_axis_angle(Vector3::unit_z(), cgmath::Rad(cam_desc.pitch));
            let q_roll =
                Quaternion::from_axis_angle(Vector3::unit_x(), cgmath::Rad(cam_desc.roll));
            let rotation = Rotation((q_yaw * q_pitch * q_roll).normalize());

            if let Err(e) = ecs.attach_component(&entity, rotation) {
                warn!("Failed to attach Rotation for camera: {:?}", e);
                continue;
            }

            let desc = CameraDescriptorEcs {
                label: cam_desc.label.clone(),
                aspect: cam_desc.aspect,
                fovy: cgmath::Rad(cam_desc.fovy),
                near: cam_desc.near,
                far: cam_desc.far,
                global_gamma: cam_desc.global_gamma,
            };
            if let Err(e) = ecs.attach_component(&entity, desc) {
                warn!("Failed to attach CameraDescriptorEcs: {:?}", e);
                continue;
            }

            // Realize GPU camera immediately
            let gpu_camera = Camera::new(
                pos.0,
                rotation.0,
                cam_desc.fovy,
                cam_desc.aspect,
                cam_desc.near,
                cam_desc.far,
                cam_desc.global_gamma,
                &device,
                &queue,
            );
            if let Err(e) = ecs.attach_component(&entity, CameraRealization) {
                warn!("Failed to attach CameraRealization marker: {:?}", e);
                continue;
            }

            // Store GPU camera in EcsCameraStore
            if let Some(mut store) = ecs.get_resource_mut::<EcsCameraStore>() {
                store.insert(entity.index, Arc::new(std::sync::RwLock::new(gpu_camera)));
            }

            if let Err(e) = ecs.attach_component(&entity, CameraDirty(true)) {
                warn!("Failed to attach CameraDirty: {:?}", e);
            }

            // Set as active camera
            ecs.insert_resource(ActiveCamera(entity));
        }
    }
}
