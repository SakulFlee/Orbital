use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

use crate::{NodeLibrary, ShaderNode};

/// A registry of named [`ShaderNode`]s, shared via `Arc`.
///
/// Nodes are keyed by name so shaders can reference them without wiring each
/// node manually. The process-wide [`NodeRegistry::global`] is preloaded with
/// the engine's built-in prelude; additional libraries are registered via
/// [`register_global_library`] at engine startup.
#[derive(Debug, Default, Clone)]
pub struct NodeRegistry {
    nodes: HashMap<Arc<str>, Arc<ShaderNode>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers every node in the given library.
    ///
    /// Registering a node whose name already maps to an *identical* node is a
    /// no-op (idempotent). Registering the same name with a *different* node
    /// returns a [`crate::ShaderPreprocessorError::ConflictingNodeName`].
    pub fn register_library(
        &mut self,
        lib: &NodeLibrary,
    ) -> Result<(), crate::ShaderPreprocessorError> {
        for node in &lib.nodes {
            self.register(node.clone())?;
        }
        Ok(())
    }

    /// Registers a single node.
    ///
    /// See [`NodeRegistry::register_library`] for the idempotency / conflict
    /// semantics.
    pub fn register(
        &mut self,
        node: Arc<ShaderNode>,
    ) -> Result<(), crate::ShaderPreprocessorError> {
        match self.nodes.get(&node.name) {
            Some(existing) if existing.source == node.source => Ok(()),
            Some(_) => Err(crate::ShaderPreprocessorError::ConflictingNodeName {
                name: node.name.to_string(),
            }),
            None => {
                self.nodes.insert(node.name.clone(), node);
                Ok(())
            }
        }
    }

    /// Looks up a node by name.
    pub fn get(&self, name: &str) -> Option<Arc<ShaderNode>> {
        self.nodes.get(name).cloned()
    }

    /// Number of registered nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns a snapshot (clone) of the process-wide registry.
    ///
    /// The global registry is lazily initialized with the engine's built-in
    /// prelude, then extended via [`register_global_library`] before any
    /// shader assembly occurs.
    pub fn global() -> Self {
        GLOBAL.read().unwrap().clone()
    }
}

/// The process-wide registry, backed by a `RwLock` for extensibility.
///
/// Starts empty. Libraries (math, engine, PBR, world-environment) are
/// registered via [`register_global_library`] at engine startup, before any
/// shader assembly occurs.
static GLOBAL: LazyLock<RwLock<NodeRegistry>> = LazyLock::new(|| RwLock::new(NodeRegistry::new()));

/// Registers a [`NodeLibrary`] with the process-wide global registry.
///
/// Call this at engine startup (before any `MaterialShader::from_descriptor`)
/// to make the library's nodes available to all shaders. Registering a node
/// whose name already maps to an identical node is idempotent; registering
/// a different node with the same name returns an error.
pub fn register_global_library(lib: &NodeLibrary) -> Result<(), crate::ShaderPreprocessorError> {
    GLOBAL.write().unwrap().register_library(lib)
}
