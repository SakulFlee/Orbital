use anyhow::Result;
use std::path::PathBuf;

/// Returns the Orbital cache root directory.
///
/// This mirrors the engine's IBL cache location (`dirs::cache_dir()` + "Orbital"):
/// - Linux:   `~/.cache/Orbital`
/// - macOS:   `~/Library/Caches/Orbital`
/// - Windows: `%LOCALAPPDATA%\Orbital`
pub fn orbital_cache_dir() -> Result<PathBuf> {
    dirs::cache_dir()
        .map(|p| p.join("Orbital"))
        .ok_or_else(|| anyhow::anyhow!("Could not determine the cache directory for this platform"))
}

/// Directory for the Orbital-owned Android SDK.
pub fn android_sdk_dir() -> Result<PathBuf> {
    Ok(orbital_cache_dir()?.join("android-sdk"))
}

/// Directory for an Orbital-owned JDK/JRE of the given version.
pub fn java_dir(version: &str) -> Result<PathBuf> {
    Ok(orbital_cache_dir()?.join("jdk").join(version))
}
