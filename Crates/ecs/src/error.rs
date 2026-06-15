use std::{error::Error, fmt::Display};

use crate::Entity;

#[derive(Debug)]
pub enum ECSError {
    InvalidEntity(Entity),
    ComponentStoreNotExisting,
}

impl Display for ECSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ECSError::InvalidEntity(entity) => {
                writeln!(f, "Invalid Entity (Index: {}, Generation: {}). Index invalid or generation was superseeded!", entity.index, entity.generation).unwrap()
            },
            ECSError::ComponentStoreNotExisting => {
                let _ = writeln!(f, "There is no ComponentStore for the requested Component!");
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
        println!("{output}");
    }
}
