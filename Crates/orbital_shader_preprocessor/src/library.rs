use std::sync::Arc;

use crate::ShaderNode;

/// A named, ordered set of [`ShaderNode`]s, registerable with a
/// [`crate::NodeRegistry`].
///
/// This is the extension point for third-party crates: expose a `NodeLibrary`
/// of your own WGSL nodes and register it, and any shader can reference those
/// nodes by name without manually adding each one.
#[derive(Debug, Clone, Default)]
pub struct NodeLibrary {
    /// Human-readable name of the library (used in diagnostics).
    pub name: String,
    /// The nodes provided by this library.
    pub nodes: Vec<Arc<ShaderNode>>,
}

impl NodeLibrary {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nodes: Vec::new(),
        }
    }

    /// Adds a node to this library.
    pub fn add(&mut self, node: ShaderNode) -> &mut Self {
        self.nodes.push(Arc::new(node));
        self
    }
}
