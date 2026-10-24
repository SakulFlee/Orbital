//! Field-editing registry for the debug UI.
//!
//! [`orbital_ecs_bridge`] owns the parsers and the [`EditableFields`] trait so
//! component crates can derive or implement them without depending on the
//! debug UI. This module is only the type-erased bridge the ECS panel uses to
//! reach those implementations through a `TypeId`.

pub use orbital_ecs_bridge::{
    EditableField, EditableFields, EditableOps, EditableValue, apply_to, display_for,
    editable_label, editable_ops, is_editable,
};

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use orbital_ecs_bridge::{CameraDescriptor, LightDescriptor, ModelDescriptor, ModelInstances};

/// Type-erased operations for one [`EditableFields`] implementation.
#[derive(Clone, Copy)]
pub struct EditableFieldsOps {
    /// Reads the component's editable fields.
    pub fields: fn(&dyn Any) -> Vec<EditableField>,
    /// Writes one field back into the component.
    pub apply: fn(&mut dyn Any, &str, &str) -> Result<(), String>,
}

fn registry() -> &'static RwLock<HashMap<TypeId, EditableFieldsOps>> {
    static REGISTRY: OnceLock<RwLock<HashMap<TypeId, EditableFieldsOps>>> = OnceLock::new();

    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Makes a component's derived fields visible to the inspector.
///
/// Engine components are registered automatically. Applications can call this
/// for their own components before the debug UI is first shown.
pub fn register_editable_fields<T: EditableFields + 'static>() {
    registry().write().expect("field registry poisoned").insert(
        TypeId::of::<T>(),
        EditableFieldsOps {
            fields: read_fields::<T>,
            apply: apply_field::<T>,
        },
    );
}

/// The field operations for `type_id`, if the type was registered.
pub fn editable_fields_ops(type_id: TypeId) -> Option<EditableFieldsOps> {
    register_builtin_editable_fields();

    registry()
        .read()
        .expect("field registry poisoned")
        .get(&type_id)
        .copied()
}

/// Registers the engine components that support per-field editing.
///
/// Idempotent, and called lazily the first time field operations are looked up,
/// so unit tests and early views cannot miss the built-ins.
pub fn register_builtin_editable_fields() {
    static BUILTINS: OnceLock<()> = OnceLock::new();

    BUILTINS.get_or_init(|| {
        register_editable_fields::<CameraDescriptor>();
        register_editable_fields::<ModelDescriptor>();
        register_editable_fields::<LightDescriptor>();
        register_editable_fields::<ModelInstances>();
    });
}

fn read_fields<T: EditableFields + 'static>(value: &dyn Any) -> Vec<EditableField> {
    value
        .downcast_ref::<T>()
        .map(EditableFields::editable_fields)
        .unwrap_or_default()
}

fn apply_field<T: EditableFields + 'static>(
    value: &mut dyn Any,
    id: &str,
    input: &str,
) -> Result<(), String> {
    value
        .downcast_mut::<T>()
        .ok_or_else(|| "component type changed under the inspector".to_string())?
        .set_editable_field(id, input)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent {
        value: f32,
    }

    impl EditableFields for TestComponent {
        fn editable_fields(&self) -> Vec<EditableField> {
            vec![EditableField {
                id: "value".to_string(),
                label: "value".to_string(),
                value: self.value.to_edit_string(),
            }]
        }

        fn set_editable_field(&mut self, id: &str, input: &str) -> Result<(), String> {
            if id != "value" {
                return Err("unknown field".to_string());
            }

            self.value.set_from_str(input)
        }
    }

    #[test]
    fn registered_fields_can_be_read_and_applied() {
        register_editable_fields::<TestComponent>();

        let mut component = TestComponent { value: 1.5 };
        let ops = editable_fields_ops(TypeId::of::<TestComponent>()).expect("registered");

        assert_eq!((ops.fields)(&component)[0].value, "1.5");
        (ops.apply)(&mut component, "value", "2.5").expect("apply failed");
        assert_eq!(component.value, 2.5);
    }

    #[test]
    fn unknown_types_have_no_field_ops() {
        assert!(editable_fields_ops(TypeId::of::<String>()).is_none());
    }
}
