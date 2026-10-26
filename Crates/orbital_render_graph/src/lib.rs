//! A small frame-graph scheduler for Orbital's renderer.
//!
//! Passes declare the resources they read and write. [`RenderGraph::compile`]
//! builds a dependency order, and [`RenderGraph::execute`] runs the pass
//! callback in that order while measuring CPU time per pass.
//!
//! This is deliberately a *frame* graph, not a shader graph. It does not try to
//! expose Vulkan subpasses (wgpu does not expose them); its job is to make pass
//! dependencies explicit and to give the renderer one place to schedule and
//! inspect the frame.

use std::collections::{BTreeSet, HashMap};
use std::fmt::Debug;
use std::hash::Hash;
use std::time::{Duration, Instant};

use wgpu::{CommandEncoder, Device, Queue};

mod texture_pool;
pub use texture_pool::{PooledTexture, TexturePool};

/// Identifies a resource in the graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub u64);

impl ResourceId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Per-pass CPU timing recorded by [`RenderGraph::execute`].
#[derive(Clone, Copy, Debug)]
pub struct PassTiming<P> {
    pub pass: P,
    pub name: &'static str,
    pub cpu: Duration,
}

/// Recording context handed to a pass callback.
pub struct PassContext<'a> {
    pub encoder: &'a mut CommandEncoder,
    pub device: &'a Device,
    pub queue: &'a Queue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphError {
    /// The graph has not been compiled yet.
    NotCompiled,
    /// The declared read/write dependencies contain a cycle.
    Cycle,
}

struct PassNode<P> {
    pass: P,
    name: &'static str,
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
    cpu: Duration,
}

/// Dependency-ordered frame graph.
pub struct RenderGraph<P> {
    passes: Vec<PassNode<P>>,
    order: Vec<usize>,
    edges: Vec<Vec<usize>>,
    compiled: bool,
    timings: Vec<PassTiming<P>>,
}

impl<P> Default for RenderGraph<P> {
    fn default() -> Self {
        Self {
            passes: Vec::new(),
            order: Vec::new(),
            edges: Vec::new(),
            compiled: false,
            timings: Vec::new(),
        }
    }
}

impl<P: Copy + Debug + Eq + Hash> RenderGraph<P> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear the previous frame's passes while retaining allocation capacity.
    pub fn begin_frame(&mut self) {
        self.passes.clear();
        self.order.clear();
        self.edges.clear();
        self.timings.clear();
        self.compiled = false;
    }

    /// Register a pass and its resource access set.
    pub fn add_pass(
        &mut self,
        pass: P,
        name: &'static str,
        reads: &[ResourceId],
        writes: &[ResourceId],
    ) {
        self.passes.push(PassNode {
            pass,
            name,
            reads: reads.to_vec(),
            writes: writes.to_vec(),
            cpu: Duration::ZERO,
        });
        self.compiled = false;
    }

    /// Build the dependency order.
    pub fn compile(&mut self) -> Result<(), GraphError> {
        let pass_count = self.passes.len();
        let mut edges = vec![Vec::new(); pass_count];
        let mut last_writer: HashMap<ResourceId, usize> = HashMap::new();
        let mut readers: HashMap<ResourceId, Vec<usize>> = HashMap::new();

        for (index, pass) in self.passes.iter().enumerate() {
            for &read in &pass.reads {
                if let Some(&writer) = last_writer.get(&read) {
                    edges[writer].push(index);
                }
                readers.entry(read).or_default().push(index);
            }

            for &write in &pass.writes {
                // Write-after-write: the previous writer must finish first.
                if let Some(&writer) = last_writer.get(&write) {
                    edges[writer].push(index);
                }
                // Write-after-read: all earlier readers must finish first.
                if let Some(current_readers) = readers.get(&write) {
                    for &reader in current_readers {
                        if reader != index {
                            edges[reader].push(index);
                        }
                    }
                }
                last_writer.insert(write, index);
                readers.insert(write, Vec::new());
            }
        }

        for outgoing in &mut edges {
            outgoing.sort_unstable();
            outgoing.dedup();
        }

        let mut indegree = vec![0usize; pass_count];
        for outgoing in &edges {
            for &target in outgoing {
                indegree[target] += 1;
            }
        }

        let mut ready: BTreeSet<usize> = indegree
            .iter()
            .enumerate()
            .filter_map(|(index, &degree)| (degree == 0).then_some(index))
            .collect();

        let mut order = Vec::with_capacity(pass_count);
        while let Some(&index) = ready.iter().next() {
            ready.remove(&index);
            order.push(index);
            for &target in &edges[index] {
                indegree[target] -= 1;
                if indegree[target] == 0 {
                    ready.insert(target);
                }
            }
        }

        if order.len() != pass_count {
            return Err(GraphError::Cycle);
        }

        self.order = order;
        self.edges = edges;
        self.compiled = true;
        Ok(())
    }

    /// Execute the compiled graph. The callback records each pass's commands.
    pub fn execute(
        &mut self,
        encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        mut run: impl FnMut(P, &mut PassContext<'_>),
    ) -> Result<&[PassTiming<P>], GraphError> {
        if !self.compiled {
            return Err(GraphError::NotCompiled);
        }

        self.timings.clear();
        for &index in &self.order {
            let start = Instant::now();
            let pass = &mut self.passes[index];
            let pass_id = pass.pass;
            {
                let mut context = PassContext {
                    encoder,
                    device,
                    queue,
                };
                run(pass_id, &mut context);
            }
            pass.cpu = start.elapsed();
            self.timings.push(PassTiming {
                pass: pass_id,
                name: pass.name,
                cpu: pass.cpu,
            });
        }

        Ok(&self.timings)
    }

    /// Compiled execution order, as indices into the insertion order.
    pub fn order(&self) -> &[usize] {
        &self.order
    }

    /// Passes in compiled execution order, with their debug names.
    pub fn ordered_passes(&self) -> impl Iterator<Item = (P, &'static str)> + '_ {
        self.order
            .iter()
            .map(|&index| (self.passes[index].pass, self.passes[index].name))
    }

    pub fn timings(&self) -> &[PassTiming<P>] {
        &self.timings
    }

    pub fn is_compiled(&self) -> bool {
        self.compiled
    }

    pub fn debug_dump(&self) -> String {
        let mut output = String::new();
        output.push_str("RenderGraph\n");
        output.push_str("  insertion order:\n");
        for (index, pass) in self.passes.iter().enumerate() {
            output.push_str(&format!(
                "    {index}: {} ({:?}) reads={:?} writes={:?}\n",
                pass.name, pass.pass, pass.reads, pass.writes
            ));
        }
        output.push_str("  compiled order:\n");
        for &index in &self.order {
            let pass = &self.passes[index];
            output.push_str(&format!("    {index}: {}\n", pass.name));
        }
        output.push_str("  timings:\n");
        for timing in &self.timings {
            output.push_str(&format!(
                "    {}: {:.3} ms\n",
                timing.name,
                timing.cpu.as_secs_f64() * 1000.0
            ));
        }
        output
    }

    pub fn to_dot(&self) -> String {
        let mut output = String::from("digraph RenderGraph {\n");
        for (index, pass) in self.passes.iter().enumerate() {
            output.push_str(&format!(
                "  n{index} [label=\"{}\"];\n",
                pass.name.replace('"', "\\\"")
            ));
        }
        for (source, outgoing) in self.edges.iter().enumerate() {
            for &target in outgoing {
                output.push_str(&format!("  n{source} -> n{target};\n"));
            }
        }
        output.push_str("}\n");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum TestPass {
        A,
        B,
    }

    fn graph_with(
        passes: &[(TestPass, &'static str, &[ResourceId], &[ResourceId])],
    ) -> RenderGraph<TestPass> {
        let mut graph = RenderGraph::new();
        for &(pass, name, reads, writes) in passes {
            graph.add_pass(pass, name, reads, writes);
        }
        graph
    }

    #[test]
    fn orders_writer_before_reader() {
        let resource = ResourceId::new(1);
        let mut graph = graph_with(&[
            (TestPass::A, "a", &[], &[resource]),
            (TestPass::B, "b", &[resource], &[]),
        ]);

        graph.compile().unwrap();

        assert_eq!(graph.order(), &[0, 1]);
    }

    #[test]
    fn orders_reader_before_later_writer() {
        let resource = ResourceId::new(1);
        let mut graph = graph_with(&[
            (TestPass::A, "a", &[resource], &[]),
            (TestPass::B, "b", &[], &[resource]),
        ]);

        graph.compile().unwrap();

        assert_eq!(graph.order(), &[0, 1]);
    }

    #[test]
    fn independent_passes_keep_insertion_order() {
        let mut graph = graph_with(&[(TestPass::A, "a", &[], &[]), (TestPass::B, "b", &[], &[])]);

        graph.compile().unwrap();

        assert_eq!(graph.order(), &[0, 1]);
    }

    #[test]
    fn begin_frame_clears_passes() {
        let resource = ResourceId::new(1);
        let mut graph = graph_with(&[(TestPass::A, "a", &[], &[resource])]);
        graph.compile().unwrap();
        graph.begin_frame();

        assert!(graph.order().is_empty());
        assert!(!graph.is_compiled());
    }

    #[test]
    fn debug_dump_lists_passes() {
        let resource = ResourceId::new(1);
        let mut graph = graph_with(&[(TestPass::A, "shadow", &[], &[resource])]);
        graph.compile().unwrap();

        let dump = graph.debug_dump();
        assert!(dump.contains("shadow"));
        assert!(dump.contains("compiled order"));
    }
}
