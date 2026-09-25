use std::sync::Arc;

/// A named unit of WGSL source that can be chained into a shader.
///
/// A node is the atomic building block of the shader-graph system: each
/// reusable function, constant or struct is defined by one `ShaderNode`, and
/// nodes are chained together (via [`crate::ShaderBuilder`]) to assemble a
/// full shader.
///
/// Nodes hold **owned** source (`Arc<str>`), so they work equally well for
/// compile-time embedded WGSL (e.g. `include_str!`) and runtime-generated
/// source (e.g. a shader built programmatically or by a third-party crate).
/// The `Arc` allows a single node to be shared cheaply across many shader
/// builders and stored in a [`crate::NodeRegistry`] without copying the WGSL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderNode {
    /// Unique name. Used for de-duplication, dependency resolution and (in the
    /// future) visualization in a node graph.
    pub name: Arc<str>,
    /// The WGSL source for this node.
    pub source: Arc<str>,
    /// Names of other nodes this one depends on. Dependencies are emitted
    /// before the dependent, transitively.
    pub depends_on: Vec<Arc<str>>,
}

impl ShaderNode {
    /// Creates a node with the given name and WGSL source.
    ///
    /// Accepts anything that converts into `Arc<str>` (`&str`, `String`,
    /// `Arc<str>`), so both `include_str!` and runtime strings work.
    pub fn new(name: impl Into<Arc<str>>, source: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
            depends_on: Vec::new(),
        }
    }

    /// Adds the given node names as dependencies of this node.
    pub fn with_deps(mut self, deps: impl IntoIterator<Item = impl Into<Arc<str>>>) -> Self {
        self.depends_on = deps.into_iter().map(Into::into).collect();
        self
    }
}
