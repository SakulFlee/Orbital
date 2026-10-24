//! Per-field editing metadata for components.
//!
//! The whole-component editor in [`crate::editable`] is fine for small
//! newtypes, but engine components such as `CameraDescriptor` have several
//! useful primitives. [`EditableFields`] exposes those fields individually so
//! the debug UI can render one text box per value.

use cgmath::{Deg, Euler, Quaternion, Rad};
use ulid::Ulid;

use crate::{EditableValue, ModelInstances};

/// One editable field of a component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditableField {
    /// Stable identifier used by the edit messages.
    pub id: String,
    /// Human-readable field name.
    pub label: String,
    /// Current value, already formatted for a text box.
    pub value: String,
}

/// A component whose primitive/string fields can be edited individually.
///
/// Implement this manually or through `#[derive(EditableFields)]` (re-exported
/// by this crate). Unsupported fields should be skipped, for example with
/// `#[editable(skip)]`.
pub trait EditableFields {
    /// The currently editable fields, in display order.
    fn editable_fields(&self) -> Vec<EditableField>;

    /// Parses `input` into the field identified by `id`.
    ///
    /// The value must be left untouched when parsing fails.
    fn set_editable_field(&mut self, id: &str, input: &str) -> Result<(), String>;
}

/// How many instances the ModelInstances editor renders at once.
pub const MODEL_INSTANCE_EDIT_LIMIT: usize = 32;

/// Stable id for one field of one model instance.
pub fn model_instance_field_id(ulid: &Ulid, field: &str) -> String {
    format!("{ulid}/{field}")
}

/// Splits a [`model_instance_field_id`] back into its instance and field.
pub fn model_instance_field_parts(id: &str) -> Option<(Ulid, &str)> {
    let (ulid, field) = id.split_once('/')?;
    Some((Ulid::from_string(ulid).ok()?, field))
}

/// A short ULID for compact instance headers.
pub fn short_model_instance_id(ulid: &Ulid) -> String {
    ulid.to_string().chars().take(8).collect()
}

/// Formats a quaternion as Euler angles in degrees.
pub fn euler_degrees_text(rotation: &Quaternion<f32>) -> String {
    let euler = Euler::<Rad<f32>>::from(*rotation);
    format!(
        "{:.3} {:.3} {:.3}",
        euler.x.0.to_degrees(),
        euler.y.0.to_degrees(),
        euler.z.0.to_degrees()
    )
}

/// Parses three Euler angles in degrees into a quaternion.
pub fn set_euler_degrees_from_str(
    rotation: &mut Quaternion<f32>,
    input: &str,
) -> Result<(), String> {
    let values = crate::editable::parse_f32_list(input, 3)?;

    *rotation = Quaternion::from(Euler {
        x: Deg(values[0]),
        y: Deg(values[1]),
        z: Deg(values[2]),
    });

    Ok(())
}

impl ModelInstances {
    /// Adds an instance with an identity transform.
    pub fn add_identity_instance(&mut self) -> Ulid {
        self.add_instance(orbital_math::Transform::identity())
    }
}

impl EditableFields for ModelInstances {
    fn editable_fields(&self) -> Vec<EditableField> {
        let mut entries = self.0.iter().collect::<Vec<_>>();
        entries.sort_by_key(|(ulid, _)| **ulid);

        let mut fields = Vec::with_capacity(entries.len() * 3);
        for (ulid, transform) in entries {
            let short = short_model_instance_id(ulid);

            fields.push(EditableField {
                id: model_instance_field_id(ulid, "position"),
                label: format!("{short} position"),
                value: transform.position.to_edit_string(),
            });
            fields.push(EditableField {
                id: model_instance_field_id(ulid, "rotation"),
                label: format!("{short} rotation (deg)"),
                value: euler_degrees_text(&transform.rotation),
            });
            fields.push(EditableField {
                id: model_instance_field_id(ulid, "scale"),
                label: format!("{short} scale"),
                value: transform.scale.to_edit_string(),
            });
        }

        fields
    }

    fn set_editable_field(&mut self, id: &str, input: &str) -> Result<(), String> {
        let (ulid, field) = model_instance_field_parts(id)
            .ok_or_else(|| format!("`{id}` is not a model instance field"))?;
        let transform = self
            .0
            .get_mut(&ulid)
            .ok_or_else(|| format!("model instance {ulid} no longer exists"))?;

        match field {
            "position" => transform.position.set_from_str(input),
            "rotation" => set_euler_degrees_from_str(&mut transform.rotation, input),
            "scale" => transform.scale.set_from_str(input),
            _ => Err(format!("unknown model instance field `{field}`")),
        }
    }
}

impl ModelInstances {
    /// Removes an instance addressed by its ULID's integer representation.
    pub fn remove_instance_by_id(&mut self, id: u128) -> Option<orbital_math::Transform> {
        self.remove_instance(&Ulid::from(id))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use cgmath::{InnerSpace, Vector3};
    use orbital_mesh::MeshDescriptor;

    use super::*;
    use crate::{CameraDescriptor, ModelDescriptor};

    #[test]
    fn the_camera_derive_exposes_every_primitive_field() {
        let camera = CameraDescriptor::default();
        let ids = camera
            .editable_fields()
            .into_iter()
            .map(|field| field.id)
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec!["label", "aspect", "fovy", "near", "far", "global_gamma"]
        );
    }

    #[test]
    fn skipped_descriptor_fields_are_not_editable() {
        let descriptor = ModelDescriptor {
            label: "helmet".to_string(),
            mesh: Arc::new(MeshDescriptor::new(Vec::new(), Vec::new())),
            materials: Vec::new(),
        };

        assert_eq!(descriptor.editable_fields().len(), 1);
        assert_eq!(descriptor.editable_fields()[0].id, "label");
    }

    #[test]
    fn model_instance_fields_round_trip_position_and_euler_rotation() {
        let mut instances = ModelInstances::new();
        let id = instances.add_identity_instance();

        assert_eq!(instances.editable_fields().len(), 3);

        instances
            .set_editable_field(&model_instance_field_id(&id, "position"), "1 2 3")
            .expect("position edit failed");
        instances
            .set_editable_field(&model_instance_field_id(&id, "rotation"), "0 0 90")
            .expect("rotation edit failed");

        let transform = instances.0[&id];
        assert!((transform.position - Vector3::new(1.0, 2.0, 3.0)).magnitude() < 1e-5);

        let euler = Euler::<Rad<f32>>::from(transform.rotation);
        assert!((euler.z.0.to_degrees() - 90.0).abs() < 1e-3);
    }

    #[test]
    fn model_instances_can_be_removed_by_integer_id() {
        let mut instances = ModelInstances::new();
        let id = instances.add_identity_instance();

        assert!(instances.remove_instance_by_id(id.0).is_some());
        assert!(instances.0.is_empty());
    }
}
