//! Component editing helpers shared by the ECS panel.

use std::any::TypeId;

use orbital_ecs::{Entity, World};
use orbital_ecs_bridge::{
    CameraDescriptor, CameraDirty, LightDescriptor, LightDirty, ModelDescriptor, ModelDirty,
    ModelInstances,
};

use crate::editable::{apply_to, editable_fields_ops, editable_ops};

/// Identifies the value a message refers to.
///
/// `field` is empty for whole-component editors such as `Position`; derived
/// components use the field id produced by `EditableFields`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldKey {
    pub entity: Entity,
    pub type_id: TypeId,
    pub field: String,
}

impl FieldKey {
    pub fn new(entity: Entity, type_id: TypeId, field: impl Into<String>) -> Self {
        Self {
            entity,
            type_id,
            field: field.into(),
        }
    }
}

/// The last path segment of a type name, e.g. `Position` for
/// `my_game::components::Position`.
pub fn short_type_name(name: &str) -> String {
    name.rsplit("::").next().unwrap_or(name).to_string()
}

/// Writes `input` into the value `key` refers to.
///
/// Reports failures instead of panicking, so a bad value leaves the component
/// untouched and the panel can show the message. Successful edits also mark the
/// entity's realization dirty flag so camera/model/light state is rebuilt.
pub fn apply_edit(world: &mut World, key: &FieldKey, input: &str) -> Result<(), String> {
    if !world.is_valid(&key.entity) {
        return Err(format!(
            "Entity {{ {} }} no longer exists",
            key.entity.index
        ));
    }

    if !key.field.is_empty()
        && let Some(ops) = editable_fields_ops(key.type_id)
    {
        let mut handle = world
            .component_mut_any(key.type_id, &key.entity)
            .ok_or_else(|| "the component is no longer attached".to_string())?;

        let result = (ops.apply)(handle.as_any_mut(), &key.field, input);
        drop(handle);
        result?;
        mark_related_dirty(world, key.entity);
        return Ok(());
    }

    editable_ops(key.type_id).ok_or_else(|| "this component type is not editable".to_string())?;

    let mut handle = world
        .component_mut_any(key.type_id, &key.entity)
        .ok_or_else(|| "the component is no longer attached".to_string())?;

    let result = apply_to(key.type_id, handle.as_any_mut(), input);
    drop(handle);
    result?;
    mark_related_dirty(world, key.entity);

    Ok(())
}

/// Marks the realization flags relevant to the entity's role.
///
/// Editing a camera's position should make the camera realize; editing a
/// model's transform should make the model realize. Deriving the flags from
/// the components the entity actually has avoids attaching unrelated ones.
pub fn mark_related_dirty(world: &mut World, entity: Entity) {
    if world.entity_has_component(TypeId::of::<CameraDescriptor>(), &entity) {
        let _ = world.attach_component(&entity, CameraDirty(true));
    }

    if world.entity_has_component(TypeId::of::<ModelDescriptor>(), &entity)
        || world.entity_has_component(TypeId::of::<ModelInstances>(), &entity)
    {
        let _ = world.attach_component(&entity, ModelDirty(true));
    }

    if world.entity_has_component(TypeId::of::<LightDescriptor>(), &entity) {
        let _ = world.attach_component(&entity, LightDirty(true));
    }
}

/// Adds an identity-transform instance to `entity`'s `ModelInstances`.
pub fn add_model_instance(world: &mut World, entity: Entity) {
    if let Some(mut handle) = world.component_mut_any(TypeId::of::<ModelInstances>(), &entity)
        && let Some(instances) = handle.as_any_mut().downcast_mut::<ModelInstances>()
    {
        instances.add_identity_instance();
    }

    mark_related_dirty(world, entity);
}

/// Removes `ulid` from `entity`'s `ModelInstances`.
pub fn remove_model_instance(world: &mut World, entity: Entity, ulid: u128) {
    if let Some(mut handle) = world.component_mut_any(TypeId::of::<ModelInstances>(), &entity)
        && let Some(instances) = handle.as_any_mut().downcast_mut::<ModelInstances>()
    {
        instances.remove_instance_by_id(ulid);
    }

    mark_related_dirty(world, entity);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marked_world() -> (World, Entity) {
        let mut world = World::new();
        let entity = world.spawn_entity();
        world
            .attach_component(&entity, CameraDescriptor::default())
            .expect("camera attach failed");
        (world, entity)
    }

    #[test]
    fn a_derived_field_edit_writes_and_marks_the_camera_dirty() {
        let (mut world, entity) = marked_world();
        let key = FieldKey::new(entity, TypeId::of::<CameraDescriptor>(), "aspect");

        apply_edit(&mut world, &key, "1.5").expect("edit failed");

        let value = world
            .component_debug(TypeId::of::<CameraDescriptor>(), &entity)
            .and_then(|handle| {
                handle
                    .as_any()
                    .downcast_ref::<CameraDescriptor>()
                    .map(|camera| camera.aspect)
            });

        assert_eq!(value, Some(1.5));
        assert!(
            world
                .component_debug(TypeId::of::<CameraDirty>(), &entity)
                .and_then(|handle| {
                    handle
                        .as_any()
                        .downcast_ref::<CameraDirty>()
                        .map(|dirty| dirty.0)
                })
                .unwrap_or(false),
            "the camera should be marked dirty"
        );
    }

    #[test]
    fn a_failed_field_edit_leaves_the_value_untouched() {
        let (mut world, entity) = marked_world();
        let key = FieldKey::new(entity, TypeId::of::<CameraDescriptor>(), "aspect");

        assert!(apply_edit(&mut world, &key, "not a number").is_err());

        let aspect = world
            .component_debug(TypeId::of::<CameraDescriptor>(), &entity)
            .and_then(|handle| {
                handle
                    .as_any()
                    .downcast_ref::<CameraDescriptor>()
                    .map(|camera| camera.aspect)
            });

        assert_eq!(aspect, Some(CameraDescriptor::default().aspect));
    }
}
