use crate::entity::Entity;

pub type BoxedEntity = Box<dyn Entity + Send + Sync>;
