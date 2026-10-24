//! Process-wide shader creation counters for diagnostics.
//!
//! These are cheap atomics/locks used by the frame timing instrumentation to
//! answer whether an application is assembling the same WGSL source many
//! times. They are not part of the rendering path's correctness.

use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

/// Snapshot of shader creation counters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ShaderStats {
    /// Number of `wgpu::ShaderModule`s created.
    pub modules_created: u64,
    /// Number of distinct assembled WGSL sources seen.
    pub unique_sources: u64,
}

static MODULES_CREATED: AtomicU64 = AtomicU64::new(0);
static UNIQUE_SOURCES: LazyLock<Mutex<HashSet<u64>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub(crate) fn record_source(source: &str) {
    MODULES_CREATED.fetch_add(1, Ordering::Relaxed);

    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);

    if let Ok(mut sources) = UNIQUE_SOURCES.lock() {
        sources.insert(hasher.finish());
    }
}

/// Returns the cumulative shader creation counters.
pub fn stats() -> ShaderStats {
    let unique_sources = UNIQUE_SOURCES
        .lock()
        .map(|sources| sources.len() as u64)
        .unwrap_or(0);

    ShaderStats {
        modules_created: MODULES_CREATED.load(Ordering::Relaxed),
        unique_sources,
    }
}
