//! Runtime camera commands.
//!
//! Commands are sent through [`Events<CameraCommand>`](orbital_ecs::Events) and
//! applied before camera realization. They update existing ECS components and
//! never recreate camera entities or GPU resources.

use cgmath::{InnerSpace, Point3, Quaternion, Rad};
use orbital_camera::ProjectionMode;
use orbital_ecs::{Component, Entity, Events, World};
use orbital_ecs_bridge::{ActiveCamera, CameraDescriptor, CameraDirty, Position, Rotation};

use crate::CameraControlSettings;

/// A command that changes a camera without recreating it.
#[derive(Debug, Clone)]
pub enum CameraCommand {
    /// Set the position and/or rotation of a camera. `None` targets the active
    /// camera.
    SetTransform {
        camera: Option<Entity>,
        position: Option<Point3<f32>>,
        rotation: Option<Quaternion<f32>>,
    },
    /// Switch the projection mode.
    SetProjection {
        camera: Option<Entity>,
        projection: ProjectionMode,
    },
    /// Set a perspective camera's vertical field of view, in degrees.
    SetFov {
        camera: Option<Entity>,
        degrees: f32,
    },
    /// Set an orthographic camera's vertical view height.
    SetZoom {
        camera: Option<Entity>,
        ortho_height: f32,
    },
    /// Set the camera clip planes. Values are ordered and validated.
    SetClipPlanes {
        camera: Option<Entity>,
        near: f32,
        far: f32,
    },
    /// Enable or disable the default camera controller.
    SetControllerEnabled(bool),
    /// Make an existing camera entity active.
    SetActiveCamera(Entity),
}

/// Applies all pending [`CameraCommand`] events.
pub fn apply_camera_commands(world: &mut World) {
    let commands = world
        .get_resource_mut::<Events<CameraCommand>>()
        .map(|mut events| events.drain().collect::<Vec<_>>())
        .unwrap_or_default();

    for command in commands {
        match command {
            CameraCommand::SetTransform {
                camera,
                position,
                rotation,
            } => {
                let Some(entity) = target_camera(world, camera) else {
                    continue;
                };
                if let Some(position) = position {
                    set_component(world, entity, Position(position));
                }
                if let Some(rotation) = rotation {
                    set_component(world, entity, Rotation(rotation.normalize()));
                }
                mark_camera_dirty(world, entity);
            }
            CameraCommand::SetProjection { camera, projection } => {
                update_descriptor(world, camera, |descriptor| {
                    descriptor.projection_mode = projection;
                });
            }
            CameraCommand::SetFov { camera, degrees } => {
                update_descriptor(world, camera, |descriptor| {
                    descriptor.fovy = Rad(degrees.clamp(1.0, 179.0).to_radians());
                });
            }
            CameraCommand::SetZoom {
                camera,
                ortho_height,
            } => {
                update_descriptor(world, camera, |descriptor| {
                    descriptor.ortho_height = ortho_height.max(0.0001);
                });
            }
            CameraCommand::SetClipPlanes { camera, near, far } => {
                let near = near.max(0.0001);
                let far = far.max(near + 0.0001);
                update_descriptor(world, camera, |descriptor| {
                    descriptor.near = near;
                    descriptor.far = far;
                });
            }
            CameraCommand::SetControllerEnabled(enabled) => {
                if let Some(mut settings) = world.get_resource_mut::<CameraControlSettings>() {
                    settings.enabled = enabled;
                }
            }
            CameraCommand::SetActiveCamera(entity) => {
                world.insert_resource(ActiveCamera(entity));
                mark_camera_dirty(world, entity);
            }
        }
    }
}

fn target_camera(world: &World, requested: Option<Entity>) -> Option<Entity> {
    requested.or_else(|| world.get_resource::<ActiveCamera>().map(|active| active.0))
}

fn update_descriptor(
    world: &mut World,
    camera: Option<Entity>,
    update: impl FnOnce(&mut CameraDescriptor),
) {
    let Some(entity) = target_camera(world, camera) else {
        return;
    };
    let Some(mut store) = world.get_component_store_mut::<CameraDescriptor>() else {
        return;
    };
    let Some(index) = store.sparse.get(entity.index).copied().flatten() else {
        return;
    };
    update(&mut store.components[index]);
    drop(store);
    mark_camera_dirty(world, entity);
}

fn set_component<T: Component>(world: &mut World, entity: Entity, component: T) {
    if let Some(mut store) = world.get_component_store_mut::<T>()
        && let Some(index) = store.sparse.get(entity.index).copied().flatten()
    {
        store.components[index] = component;
        return;
    }

    let _ = world.attach_component(&entity, component);
}

fn mark_camera_dirty(world: &mut World, entity: Entity) {
    if let Some(mut store) = world.get_component_store_mut::<CameraDirty>()
        && let Some(index) = store.sparse.get(entity.index).copied().flatten()
    {
        store.components[index].0 = true;
        return;
    }

    let _ = world.attach_component(&entity, CameraDirty(true));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera_world() -> (World, Entity) {
        let mut world = World::new();
        let entity = world.spawn_entity();
        world
            .attach_component(&entity, CameraDescriptor::default())
            .unwrap();
        world
            .attach_component(&entity, Position(Point3::new(0.0, 0.0, 0.0)))
            .unwrap();
        world
            .attach_component(&entity, Rotation::identity())
            .unwrap();
        world.attach_component(&entity, CameraDirty(false)).unwrap();
        world.insert_resource(ActiveCamera(entity));
        world.insert_resource(CameraControlSettings::default());
        world.insert_resource(Events::<CameraCommand>::new());
        (world, entity)
    }

    #[test]
    fn commands_update_the_active_camera_without_recreating_it() {
        let (mut world, entity) = camera_world();

        {
            let mut events = world.get_resource_mut::<Events<CameraCommand>>().unwrap();
            events.send(CameraCommand::SetProjection {
                camera: None,
                projection: ProjectionMode::Orthographic,
            });
            events.send(CameraCommand::SetZoom {
                camera: None,
                ortho_height: 12.0,
            });
            events.send(CameraCommand::SetClipPlanes {
                camera: None,
                near: 0.5,
                far: 500.0,
            });
            events.send(CameraCommand::SetTransform {
                camera: None,
                position: Some(Point3::new(1.0, 2.0, 3.0)),
                rotation: Some(Quaternion::new(0.0, 1.0, 0.0, 0.0)),
            });
        }

        apply_camera_commands(&mut world);

        let descriptor = world
            .get_component_store::<CameraDescriptor>()
            .and_then(|store| store.get_component(entity.index).cloned())
            .expect("camera descriptor");
        assert_eq!(descriptor.projection_mode, ProjectionMode::Orthographic);
        assert_eq!(descriptor.ortho_height, 12.0);
        assert_eq!(descriptor.near, 0.5);
        assert_eq!(descriptor.far, 500.0);

        let position = world
            .get_component_store::<Position>()
            .and_then(|store| store.get_component(entity.index).copied())
            .expect("position");
        assert_eq!(position.0, Point3::new(1.0, 2.0, 3.0));

        let dirty = world
            .get_component_store::<CameraDirty>()
            .and_then(|store| store.get_component(entity.index).copied())
            .expect("dirty flag");
        assert!(dirty.0);
    }

    #[test]
    fn controller_command_updates_settings() {
        let (mut world, _) = camera_world();
        world
            .get_resource_mut::<Events<CameraCommand>>()
            .unwrap()
            .send(CameraCommand::SetControllerEnabled(false));

        apply_camera_commands(&mut world);

        assert!(
            !world
                .get_resource::<CameraControlSettings>()
                .unwrap()
                .enabled
        );
    }
}
