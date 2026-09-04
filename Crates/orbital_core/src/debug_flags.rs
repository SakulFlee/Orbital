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
//! # Remove again to restore normal behaviour
//! adb shell run-as <package> rm files/orbital_disable_cull files/orbital_cull_debug
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