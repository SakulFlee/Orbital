use std::collections::HashSet;
use std::sync::Arc;

use crate::{NodeRegistry, ShaderNode};

/// Assembles a full WGSL shader from registered [`ShaderNode`]s.
///
/// Nodes are requested **by name** and resolved against a [`NodeRegistry`].
/// Dependencies are resolved transitively, each node is emitted exactly once,
/// and dependencies always precede their dependents. Additional raw WGSL
/// (e.g. an entrypoint body) can be appended after the nodes.
///
/// This replaces the old runtime directory-listing `#import` approach: all
/// node source is code-defined (via `include_str!` or runtime strings), so
/// shader assembly is fully portable (including Android).
#[derive(Debug)]
pub struct ShaderBuilder {
    registry: Arc<NodeRegistry>,
    /// Ordered, deduplicated node names resolved so far.
    order: Vec<Arc<str>>,
    /// Extra raw WGSL appended after the resolved nodes.
    extra: Vec<Arc<str>>,
    /// Names of nodes currently being visited on the resolution stack, used to
    /// detect dependency cycles.
    visiting: HashSet<Arc<str>>,
    /// When enabled, `build()` logs the ordered node list and full assembled
    /// WGSL source via `log::debug!`. Off by default.
    debug: bool,
}

impl ShaderBuilder {
    /// Creates a builder resolving nodes against the given registry.
    pub fn new(registry: Arc<NodeRegistry>) -> Self {
        Self {
            registry,
            order: Vec::new(),
            extra: Vec::new(),
            visiting: HashSet::new(),
            debug: false,
        }
    }

    /// Creates a builder resolving nodes against the process-wide global
    /// registry (preloaded with the engine prelude).
    pub fn with_global_registry() -> Self {
        Self::new(Arc::new(NodeRegistry::global().clone()))
    }

    /// Enables (or disables) `log::debug!` output of the assembled node list
    /// and full WGSL source when [`ShaderBuilder::build`] is called.
    ///
    /// This lets end-users verify that everything was included correctly.
    pub fn set_debug(&mut self, debug: bool) -> &mut Self {
        self.debug = debug;
        self
    }

    /// Whether debug output is enabled.
    pub fn debug(&self) -> bool {
        self.debug
    }

    /// Requests a node by name, resolving its transitive dependencies
    /// (depth-first, each emitted once, dependencies before dependents).
    ///
    /// Returns [`crate::ShaderPreprocessorError::UnknownNode`] if the name is
    /// not registered, or
    /// [`crate::ShaderPreprocessorError::DependencyCycle`] if resolving the
    /// node would follow a circular dependency chain.
    pub fn add_node(&mut self, name: &str) -> Result<&mut Self, crate::ShaderPreprocessorError> {
        let node =
            self.registry
                .get(name)
                .ok_or_else(|| crate::ShaderPreprocessorError::UnknownNode {
                    name: name.to_string(),
                })?;
        self.resolve(&node)?;
        Ok(self)
    }

    /// Requests several nodes by name.
    pub fn add_nodes(
        &mut self,
        names: &[&str],
    ) -> Result<&mut Self, crate::ShaderPreprocessorError> {
        for name in names {
            self.add_node(name)?;
        }
        Ok(self)
    }

    /// Appends raw WGSL after the resolved nodes (e.g. an entrypoint body that
    /// is not itself a reusable node).
    pub fn add_source(&mut self, source: impl Into<Arc<str>>) -> &mut Self {
        self.extra.push(source.into());
        self
    }

    /// Recursively resolves a node and its dependencies into `self.order`,
    /// emitting each node exactly once.
    ///
    /// `visiting` tracks nodes on the current DFS path so that circular
    /// dependencies are reported as an error instead of recursing forever.
    fn resolve(&mut self, node: &Arc<ShaderNode>) -> Result<(), crate::ShaderPreprocessorError> {
        // Guard against cycles: if this node is already on the resolution stack
        // we have a circular dependency.
        if self.visiting.contains(&node.name) {
            return Err(crate::ShaderPreprocessorError::DependencyCycle {
                name: node.name.to_string(),
            });
        }

        // Already fully resolved (and emitted) on a previous path.
        if self.order.iter().any(|o| *o == node.name) {
            return Ok(());
        }

        self.visiting.insert(node.name.clone());

        let deps = node.depends_on.clone();
        for dep in deps {
            if self.order.iter().any(|o| *o == dep) {
                continue;
            }
            if let Some(dep_node) = self.registry.get(&dep) {
                self.resolve(&dep_node)?;
            }
        }

        self.visiting.remove(&node.name);
        self.order.push(node.name.clone());
        Ok(())
    }

    /// Emits the assembled WGSL source: the resolved nodes (dependency-first,
    /// deduplicated) followed by any raw source appended via
    /// [`ShaderBuilder::add_source`].
    ///
    /// If debug output is enabled (see [`ShaderBuilder::set_debug`]), the
    /// ordered node list and full assembled source are logged via
    /// `log::debug!`.
    pub fn build(&self) -> String {
        let mut out = String::new();
        for name in &self.order {
            if let Some(node) = self.registry.get(name) {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&node.source);
            }
        }
        for extra in &self.extra {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(extra);
        }

        if self.debug {
            let nodes = self
                .order
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            log::debug!(
                "[ShaderBuilder] resolved nodes ({}): {nodes}",
                self.order.len()
            );
            log::debug!("[ShaderBuilder] assembled WGSL:\n{out}");
        }

        out
    }

    /// Returns the ordered, deduplicated list of node names that will be
    /// emitted by [`ShaderBuilder::build`]. Useful for diagnostics and the
    /// future node-graph visualization.
    pub fn resolved_order(&self) -> &[Arc<str>] {
        &self.order
    }

    /// A deduplicated set of node names (for graph introspection).
    pub fn node_names(&self) -> HashSet<&str> {
        self.order.iter().map(|s| &**s).collect()
    }
}
