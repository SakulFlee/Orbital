//! Thread-safe cache of realized GPU meshes, keyed by [`MeshDescriptor`].
//!
//! Extracted from `ModelStore` so it can be shared by both legacy and
//! ECS code paths during migration.

use std::sync::{Arc, RwLock};

use orbital_core::cache::Cache;

use crate::{Mesh, MeshDescriptor};

/// Thread-safe cache of realized GPU meshes.
pub type MeshCache = RwLock<Cache<Arc<MeshDescriptor>, Mesh>>;
