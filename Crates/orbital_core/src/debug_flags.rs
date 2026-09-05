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
//! # Disable CPU frustum culling (draw all instances)
//! adb shell run-as <package> touch files/orbital_disable_cull
//!
//! # Remove again to restore normal behaviour
//! adb shell run-as <package> rm files/orbital_disable_cull
//! ```
//!
//! Both mechanisms are read **once** and cached for the process lifetime —
//! these are launch-time diagnostics, not runtime toggles.

use std::sync::OnceLock;

use orbital_file_manager::FileManager;

/// Marker file (in the storage root) that disables CPU frustum culling.
const DISABLE_CULL_FILE: &str = "orbital_disable_cull";

/// Whether CPU frustum culling is disabled (`ORBITAL_DISABLE_CULL=1` env or an
/// existing `orbital_disable_cull` storage marker file). When true, all
/// instances are drawn unconditionally (no frustum filtering).
pub fn disable_cull() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| match std::env::var("ORBITAL_DISABLE_CULL") {
        Ok(v) => v == "1",
        Err(_) => FileManager::global()
            .map(|fm| fm.storage_path_exists(DISABLE_CULL_FILE))
            .unwrap_or(false),
    })
}

/// Log the resolved debug flags and the storage root once per process. Call
/// this each process start (idempotent) so that from logcat we can confirm
/// whether env vars / Android marker files were actually picked up.
pub fn log_active_flags() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let root = FileManager::global()
            .map(|fm| fm.storage_root().display().to_string())
            .unwrap_or_else(|_| "<FileManager not initialized>".to_string());
        crate::logging::info!(
            "debug_flags: disable_cull={} storage_root={}",
            disable_cull(),
            root,
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disable_cull_defaults_false() {
        // Without env var or marker file, disable_cull should be false.
        // (We can't easily test the true path without side-effects.)
        assert!(!disable_cull());
    }
}
