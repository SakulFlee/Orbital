//! Thread-safe cache of realized material shaders, keyed by [`MaterialShaderDescriptor`].
//!
//! Extracted from `ModelStore` so it can be shared by both legacy and
//! ECS code paths during migration.

use std::sync::{Arc, RwLock};

use orbital_core::cache::Cache;

use crate::material_shader::{MaterialShader, MaterialShaderDescriptor};

/// Thread-safe cache of realized material shaders.
pub type MaterialShaderCache = RwLock<Cache<Arc<MaterialShaderDescriptor>, MaterialShader>>;
