//! Light descriptor type.
//!
//! `LightDescriptor` contains the data that defines a light source.
//! The ECS system stores lights as `LightDescriptor` components
//! and builds a unified GPU buffer from them each frame.

mod descriptor;
pub use descriptor::*;
