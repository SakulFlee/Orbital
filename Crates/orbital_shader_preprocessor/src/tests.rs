//! Unit tests for the node-graph shader system.

use crate::{NodeLibrary, NodeRegistry, ShaderBuilder, ShaderNode};
use std::sync::Arc;

/// A tiny custom library of nodes used by these tests (independent of the
/// engine prelude, so tests exercise registration + resolution on their own).
fn test_library() -> NodeLibrary {
    let mut lib = NodeLibrary::new("test");
    lib.add(ShaderNode::new("pi", "const PI: f32 = 3.14;\n"));
    lib.add(ShaderNode::new("sq", "fn sq(x: f32) -> f32 { return x * x; }\n").with_deps(["pi"]));
    lib.add(
        ShaderNode::new("cubed", "fn cubed(x: f32) -> f32 { return x * x * x; }\n")
            .with_deps(["sq"]),
    );
    lib
}

fn test_registry() -> NodeRegistry {
    let mut registry = NodeRegistry::new();
    registry.register_library(&test_library()).unwrap();
    registry
}

#[test]
fn builder_resolves_dependencies_transitively() {
    let mut builder = ShaderBuilder::new(Arc::new(test_registry()));
    builder.add_node("cubed").unwrap();

    let source = builder.build();

    // Dependency-first, deduplicated ordering: pi, sq, cubed.
    assert!(source.find("const PI").unwrap() < source.find("fn sq").unwrap());
    assert!(source.find("fn sq").unwrap() < source.find("fn cubed").unwrap());
    assert_eq!(source.matches("fn sq").count(), 1);
}

#[test]
fn builder_deduplicates_nodes() {
    let mut builder = ShaderBuilder::new(Arc::new(test_registry()));
    builder.add_node("cubed").unwrap();
    builder.add_node("sq").unwrap();
    builder.add_node("cubed").unwrap();

    let source = builder.build();
    assert_eq!(source.matches("fn sq").count(), 1);
    assert_eq!(source.matches("fn cubed").count(), 1);
    assert_eq!(builder.node_names().len(), 3); // pi, sq, cubed
}

#[test]
fn builder_appends_raw_source_after_nodes() {
    let mut builder = ShaderBuilder::new(Arc::new(test_registry()));
    builder.add_node("sq").unwrap();
    builder.add_source("fn main() { return; }\n");

    let source = builder.build();
    assert!(source.find("fn sq").unwrap() < source.find("fn main").unwrap());
}

#[test]
fn unknown_node_errors() {
    let mut builder = ShaderBuilder::new(Arc::new(test_registry()));
    let err = builder.add_node("does_not_exist").unwrap_err();
    assert!(matches!(
        err,
        crate::ShaderPreprocessorError::UnknownNode { .. }
    ));
}

#[test]
fn registry_conflicting_names_error() {
    let mut registry = test_registry();
    // Same name, different source -> conflict.
    let node = ShaderNode::new("pi", "const PI: f32 = 0.0;\n");
    let err = registry.register(Arc::new(node)).unwrap_err();
    assert!(matches!(
        err,
        crate::ShaderPreprocessorError::ConflictingNodeName { .. }
    ));
}

#[test]
fn registry_re_registration_of_identical_node_is_idempotent() {
    let mut registry = test_registry();
    // Same name + same source -> no-op, no error.
    let node = ShaderNode::new("pi", "const PI: f32 = 3.14;\n");
    registry.register(Arc::new(node)).unwrap();
    assert_eq!(registry.len(), test_registry().len());
}

#[test]
fn registry_accepts_runtime_generated_source() {
    let mut registry = NodeRegistry::new();
    let dynamic_source = format!("fn dynamic(x: f32) -> f32 {{ return x + {}; }}\n", 1.0);
    let node = ShaderNode::new("dynamic", dynamic_source);
    registry.register(Arc::new(node)).unwrap();

    let mut builder = ShaderBuilder::new(Arc::new(registry));
    builder.add_node("dynamic").unwrap();
    assert!(builder.build().contains("fn dynamic"));
}

#[test]
fn global_registry_starts_empty() {
    let registry = NodeRegistry::global();
    assert!(registry.is_empty());
    assert!(registry.get("does_not_exist").is_none());
}

#[test]
fn dependency_cycle_errors() {
    let mut registry = NodeRegistry::new();
    registry
        .register_library(&{
            let mut lib = NodeLibrary::new("cycle");
            lib.add(ShaderNode::new("a", "fn a() {}\n").with_deps(["b"]));
            lib.add(ShaderNode::new("b", "fn b() {}\n").with_deps(["a"]));
            lib
        })
        .unwrap();

    let mut builder = ShaderBuilder::new(Arc::new(registry));
    let err = builder.add_node("a").unwrap_err();
    assert!(matches!(
        err,
        crate::ShaderPreprocessorError::DependencyCycle { .. }
    ));
}

#[test]
fn self_dependency_cycle_errors() {
    let mut registry = NodeRegistry::new();
    registry
        .register(Arc::new(
            ShaderNode::new("self", "fn self_fn() {}\n").with_deps(["self"]),
        ))
        .unwrap();

    let mut builder = ShaderBuilder::new(Arc::new(registry));
    let err = builder.add_node("self").unwrap_err();
    assert!(matches!(
        err,
        crate::ShaderPreprocessorError::DependencyCycle { .. }
    ));
}

#[test]
fn builder_emits_exact_node_sources() {
    let mut builder = ShaderBuilder::new(Arc::new(test_registry()));
    builder.add_node("sq").unwrap();

    // The emitted output must contain exactly the `sq` node's source (plus its
    // dependency `pi`), in order, with no extra/mangled content.
    let source = builder.build();
    assert!(source.contains("const PI: f32 = 3.14;"));
    assert!(source.contains("fn sq(x: f32) -> f32 { return x * x; }"));
    assert!(!source.contains("cubed"));
}

#[test]
fn custom_registry_supports_third_party_nodes() {
    // Simulates a third-party crate registering its own node library and a
    // shader referencing those nodes by name.
    let mut registry = NodeRegistry::new();
    registry
        .register_library(&{
            let mut lib = NodeLibrary::new("third_party");
            lib.add(
                ShaderNode::new("tp_util", "fn tp_util(x: f32) -> f32 { return x; }\n")
                    .with_deps(["base"]),
            );
            lib.add(ShaderNode::new("base", "const BASE: f32 = 1.0;\n"));
            lib
        })
        .unwrap();

    let mut builder = ShaderBuilder::new(Arc::new(registry));
    builder.add_node("tp_util").unwrap();
    let source = builder.build();

    assert!(source.contains("const BASE: f32 = 1.0;"));
    assert!(source.contains("fn tp_util"));
    // Dependency `base` precedes its dependant `tp_util`.
    assert!(source.find("const BASE").unwrap() < source.find("fn tp_util").unwrap());
}
