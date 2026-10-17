use std::{any::TypeId, error::Error, fmt::Display};

use crate::Entity;

#[derive(Debug)]
pub enum ECSError {
    InvalidEntity(Entity),
    ComponentStoreNotExisting,
    /// The entity does not currently hold the component, so it cannot be
    /// replaced. Use `World::insert_component_any` to attach it first.
    ComponentNotAttached(Entity),
    /// A type-erased component value did not match the type held by the store.
    ComponentTypeMismatch(TypeId),
}

impl Display for ECSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ECSError::InvalidEntity(entity) => {
                writeln!(
                    f,
                    "Invalid Entity (Index: {}, Generation: {}). Index invalid or generation was superseeded!",
                    entity.index, entity.generation
                )?;
            }
            ECSError::ComponentStoreNotExisting => {
                writeln!(f, "There is no ComponentStore for the requested Component!")?;
            }
            ECSError::ComponentNotAttached(entity) => {
                writeln!(
                    f,
                    "Entity (Index: {}, Generation: {}) does not hold the requested Component!",
                    entity.index, entity.generation
                )?;
            }
            ECSError::ComponentTypeMismatch(type_id) => {
                writeln!(
                    f,
                    "The provided value is not of the stored Component type ({type_id:?})!"
                )?;
            }
        }
        writeln!(f, "{self:?}")
    }
}

impl Error for ECSError {}

#[cfg(test)]
mod tests {
    use crate::{ECSError, Entity};

    #[test]
    fn test_error_display() {
        let error = ECSError::InvalidEntity(Entity::new(1, 123));
        let output = error.to_string();
        assert!(output.contains("Invalid Entity"));
    }
}
