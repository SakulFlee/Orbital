use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Read;
use std::path::{Path, PathBuf};

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

/// Downloads a URL to `dest` with a progress bar.
///
/// The download is written directly to `dest`. Callers that need atomicity
/// (e.g. avoiding partial files) should download to a temp path and rename.
pub fn download_with_progress(url: &str, dest: &Path, label: &str) -> Result<()> {
    let mut response = ureq::get(url)
        .call()
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

    let total = response.body().content_length().unwrap_or(0);

    let bar = ProgressBar::new(total);
    bar.set_style(
        ProgressStyle::with_template("{msg} [{bar:40}] {percent}% {bytes}/{total_bytes}")
            .unwrap()
            .progress_chars("=>-"),
    );
    bar.set_message(label.to_string());

    let mut file = std::fs::File::create(dest)
        .map_err(|e| anyhow::anyhow!("Failed to create file {}: {}", dest.display(), e))?;

    let mut reader = response.body_mut().as_reader();
    let mut buf = [0u8; 64 * 1024];

    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;
        if n == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buf[..n])
            .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
        bar.inc(n as u64);
    }

    bar.finish();
    println!();
    Ok(())
}
