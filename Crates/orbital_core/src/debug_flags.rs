//! Engine debug flags with an env-var-first, storage-file-fallback lookup.
//!
//! On desktop, environment variables set at launch work as usual:
//!
//! ```sh
//! ORBITAL_DISABLE_CULL=1 ./orbital
//! ```
//!
//! On Android, Zygote-spawned app processes do **not** inherit the shell
//! environment, so `adb shell ORBITAL_X=1 am start ...` has no effect. As a
//! workaround, flags can be placed in the app's storage root (the directory
//! handed to [`orbital_file_manager::FileManager::init_android_global`], i.e.
//! the app's files dir) as marker files — settable over adb without root:
//!
//! ```sh
//! # Disable GPU culling entirely (un-culled direct draws, like the shadow pass)
//! adb shell run-as <package> touch files/orbital_disable_cull
//!
//! # Cull debug: readback logging (any content / empty file)
//! adb shell run-as <package> touch files/orbital_cull_debug
//!
//! # Cull debug: full cull_all mode (skips frustum test + readback logging)
//! adb shell run-as <package> sh -c 'echo cull_all > files/orbital_cull_debug'
//!
//! # Single-encoder cull: submit cull compute with the render pass (barrier probe)
//! adb shell run-as <package> touch files/orbital_cull_single_encoder
//!
//! # Cull-all probe: admit every instance (skips only the frustum test)
//! adb shell run-as <package> touch files/orbital_cull_all
//!
//! # CPU-args probe: skip the cull compute entirely; CPU writes the indirect
//! draw args + identity-compacted instances. Isolates "does
//! draw_indexed_indirect work on this driver at all" from "does it work with
//! compute-written args".
//! adb shell run-as <package> touch files/orbital_cull_cpu_args
//!
//! # Remove again to restore normal behaviour
//! adb shell run-as <package> rm files/orbital_disable_cull files/orbital_cull_debug files/orbital_cull_single_encoder
//! ```
//!
//! Both mechanisms are read **once** and cached for the process lifetime —
//! these are launch-time diagnostics, not runtime toggles.

use std::sync::OnceLock;

use orbital_file_manager::FileManager;

/// Marker file (in the storage root) that disables GPU culling.
const DISABLE_CULL_FILE: &str = "orbital_disable_cull";
/// Marker file (in the storage root) holding the cull debug mode.
const CULL_DEBUG_FILE: &str = "orbital_cull_debug";
/// Marker file (in the storage root) that switches culling to single-encoder
/// mode (compute dispatched inside the render submission).
const SINGLE_ENCODER_FILE: &str = "orbital_cull_single_encoder";
/// Marker file (in the storage root) that forces culling to admit every
/// instance (`cull_all` entry point — skips only the frustum test, keeps the
/// identical compaction + indirect path). Content-free: its existence is
/// enough, so it can be enabled with a plain `touch` (no `echo`/`sh`).
const CULL_ALL_FILE: &str = "orbital_cull_all";
/// Marker file (in the storage root) enabling the CPU-args probe: the cull
/// compute is skipped entirely; indirect draw args and the compacted instance
/// data are written from the CPU instead. Content-free marker file.
const CULL_CPU_ARGS_FILE: &str = "orbital_cull_cpu_args";

/// Whether GPU culling is disabled (`ORBITAL_DISABLE_CULL=1` env or an
/// existing `orbital_disable_cull` storage marker file).
pub fn disable_cull() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| match std::env::var("ORBITAL_DISABLE_CULL") {
        Ok(v) => v == "1",
        Err(_) => FileManager::global()
            .map(|fm| fm.storage_path_exists(DISABLE_CULL_FILE))
            .unwrap_or(false),
    })
}

/// Whether GPU culling is dispatched inside the render submission
/// (`ORBITAL_CULL_SINGLE_ENCODER=1` env or an existing
/// `orbital_cull_single_encoder` storage marker file). SIDESTEPS
/// cross-submission storage→vertex/indirect barrier gaps by submitting the
/// cull compute together with the render passes.
pub fn cull_single_encoder() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| match std::env::var("ORBITAL_CULL_SINGLE_ENCODER") {
        Ok(v) => v == "1",
        Err(_) => FileManager::global()
            .map(|fm| fm.storage_path_exists(SINGLE_ENCODER_FILE))
            .unwrap_or(false),
    })
}

/// Whether culling should admit every instance (skipping only the frustum
/// test) via the `cull_all` entry point. Existence-based: `ORBITAL_CULL_ALL=1`
/// env or an `orbital_cull_all` storage marker file (plain `touch`). Keeps the
/// identical compaction + indirect path as normal culling, so it cleanly
/// separates a frustum-math problem from a compaction/indirect one.
pub fn cull_all() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| match std::env::var("ORBITAL_CULL_ALL") {
        Ok(v) => v == "1",
        Err(_) => FileManager::global()
            .map(|fm| fm.storage_path_exists(CULL_ALL_FILE))
            .unwrap_or(false),
    })
}

/// Mode for the cull-pipeline debug probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullDebugMode {
    /// Normal behaviour.
    Off,
    /// `ORBITAL_CULL_DEBUG=1` / marker file with other content: throttled
    /// readback logging of per-model visible counts + indirect args.
    Readback,
    /// `ORBITAL_CULL_DEBUG=cull_all` / marker file with `cull_all` content:
    /// skips the frustum test (every instance admitted) **and** enables
    /// readback logging.
    CullAll,
}

impl CullDebugMode {
    /// Whether readback logging is enabled in this mode.
    pub fn readback(self) -> bool {
        matches!(self, CullDebugMode::Readback | CullDebugMode::CullAll)
    }
}

/// Whether the CPU-args cull probe is active (`ORBITAL_CULL_CPU_ARGS=1` env or
/// an existing `orbital_cull_cpu_args` storage marker file). Skips the cull
/// compute entirely; the system writes indirect draw args and the compacted
/// instance data from the CPU, so the render pass consumes
/// `draw_indexed_indirect` with CPU-written args. Isolates "does the driver
/// execute `draw_indexed_indirect` with args it did not write itself".
pub fn cull_cpu_args() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| match std::env::var("ORBITAL_CULL_CPU_ARGS") {
        Ok(v) => v == "1",
        Err(_) => FileManager::global()
            .map(|fm| fm.storage_path_exists(CULL_CPU_ARGS_FILE))
            .unwrap_or(false),
    })
}

/// Parses a `ORBITAL_CULL_DEBUG` value; `None` means "not set".
fn parse_cull_debug(env: Option<&str>, file: Option<&str>) -> CullDebugMode {
    // An empty env var counts as unset; an empty marker file still counts
    // (its mere existence enables readback).
    let env = env.filter(|v| !v.is_empty());
    for candidate in [env, file].into_iter().flatten() {
        if candidate.trim().eq_ignore_ascii_case("cull_all") {
            return CullDebugMode::CullAll;
        }
        if candidate.trim() == "1" {
            return CullDebugMode::Readback;
        }
        // Marker file with arbitrary/empty content still enables readback.
        if candidate.is_empty() {
            return CullDebugMode::Readback;
        }
    }
    CullDebugMode::Off
}

/// Active [`CullDebugMode`] (`ORBITAL_CULL_DEBUG` env or the
/// `orbital_cull_debug` storage marker file, env taking precedence).
pub fn cull_debug_mode() -> CullDebugMode {
    static MODE: OnceLock<CullDebugMode> = OnceLock::new();
    *MODE.get_or_init(|| {
        let env = std::env::var("ORBITAL_CULL_DEBUG").ok();
        let file = FileManager::global()
            .ok()
            .filter(|fm| fm.storage_path_exists(CULL_DEBUG_FILE))
            .and_then(|fm| fm.read_to_string(CULL_DEBUG_FILE).ok());
        parse_cull_debug(env.as_deref(), file.as_deref())
    })
}

/// Log the resolved cull-related debug flags and the storage root once per
/// process. Call this each process start (idempotent) so that from logcat we
/// can confirm whether env vars / Android marker files were actually picked up
/// — and where marker files resolve to.
pub fn log_active_flags() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let root = FileManager::global()
            .map(|fm| fm.storage_root().display().to_string())
            .unwrap_or_else(|_| "<FileManager not initialized>".to_string());
        crate::logging::info!(
            "debug_flags: disable_cull={} cull_debug={:?} single_encoder={} cull_all={} cpu_args={} storage_root={}",
            disable_cull(),
            cull_debug_mode(),
            cull_single_encoder(),
            cull_all(),
            cull_cpu_args(),
            root,
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cull_debug_values() {
        assert_eq!(parse_cull_debug(None, None), CullDebugMode::Off);
        assert_eq!(parse_cull_debug(Some(""), None), CullDebugMode::Off);
        assert_eq!(parse_cull_debug(Some("1"), None), CullDebugMode::Readback);
        assert_eq!(
            parse_cull_debug(Some("cull_all"), None),
            CullDebugMode::CullAll
        );
        assert_eq!(
            parse_cull_debug(Some("CULL_ALL"), None),
            CullDebugMode::CullAll
        );
        // File fallback: empty marker file → readback, "cull_all" → full mode.
        assert_eq!(parse_cull_debug(None, Some("")), CullDebugMode::Readback);
        assert_eq!(
            parse_cull_debug(None, Some("cull_all\n")),
            CullDebugMode::CullAll
        );
        // Env takes precedence over the file.
        assert_eq!(
            parse_cull_debug(Some("1"), Some("cull_all")),
            CullDebugMode::Readback
        );
        // Unknown env values do not enable anything.
        assert_eq!(parse_cull_debug(Some("yes"), None), CullDebugMode::Off);
        // Readback helper.
        assert!(CullDebugMode::CullAll.readback());
        assert!(CullDebugMode::Readback.readback());
        assert!(!CullDebugMode::Off.readback());
    }
}