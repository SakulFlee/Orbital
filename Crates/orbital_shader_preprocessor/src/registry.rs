use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::{NodeLibrary, ShaderNode};

/// A registry of named [`ShaderNode`]s, shared via `Arc`.
///
/// Nodes are keyed by name so shaders can reference them without wiring each
/// node manually. The process-wide [`NodeRegistry::global`] is preloaded with
/// the engine's built-in prelude; third-party crates register their own
/// [`NodeLibrary`]s (into the global registry, or a custom one for isolation /
/// testing).
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

    /// Returns the process-wide registry, lazily initialized with the engine's
    /// built-in prelude.
    pub fn global() -> &'static NodeRegistry {
        static GLOBAL: OnceLock<NodeRegistry> = OnceLock::new();
        GLOBAL.get_or_init(|| {
            let mut registry = NodeRegistry::new();
            let prelude = crate::prelude::prelude_library();
            registry
                .register_library(&prelude)
                .expect("prelude library must not contain conflicting node names");
            registry
        })
    }
}
