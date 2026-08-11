use anyhow::{Context, Result};
use inquire::Confirm;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The Temurin LTS version we manage for Orbital projects.
pub const JDK_VERSION: &str = "25";

/// Returns the path to the `java` executable for a given JRE home directory.
fn java_executable(jre_home: &Path) -> PathBuf {
    let bin = jre_home.join("bin");
    if cfg!(windows) {
        bin.join("java.exe")
    } else {
        bin.join("java")
    }
}

/// Returns the path to the `javac` executable for a given JDK home directory.
fn javac_executable(jdk_home: &Path) -> PathBuf {
    let bin = jdk_home.join("bin");
    if cfg!(windows) {
        bin.join("javac.exe")
    } else {
        bin.join("javac")
    }
}

/// True if the given home is a full JDK (has both `java` and `javac`).
/// Gradle needs the compiler, so a bare JRE is not sufficient.
fn is_jdk(home: &Path) -> bool {
    java_executable(home).exists() && javac_executable(home).exists()
}

/// Checks the Orbital-owned JDK first, then JAVA_HOME, then PATH.
/// Returns the JDK home directory if a working Java (with javac) is found.
pub fn find_java() -> Option<PathBuf> {
    // 1. Orbital-owned JDK in the cache dir (may be nested in a versioned subdir)
    if let Ok(dir) = crate::tooling::java_dir(JDK_VERSION) {
        if let Ok(jdk_home) = find_jre_home_in(&dir) {
            let java = java_executable(&jdk_home);
            if java.exists() && java_works(&java) && is_jdk(&jdk_home) {
                return Some(jdk_home);
            }
        }
    }

    // 2. JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java_home = PathBuf::from(java_home);
        let java = java_executable(&java_home);
        if java.exists() && java_works(&java) && is_jdk(&java_home) {
            return Some(java_home);
        }
    }

    // 3. java on PATH (only if a full JDK is available)
    if java_works_on_path() {
        if let Some(java) = find_java_on_path() {
            if let Some(bin) = java.parent() {
                if let Some(home) = bin.parent() {
                    let home = home.to_path_buf();
                    if is_jdk(&home) {
                        return Some(home);
                    }
                }
            }
        }
    }

    None
}

/// Ensures a working Java is available, prompting to install one if needed.
/// Returns the JRE home directory.
pub fn ensure_java() -> Result<PathBuf> {
    if let Some(home) = find_java() {
        println!("Java found at: {}", home.display());
        return Ok(home);
    }

    println!("\nNo compatible Java runtime found (JDK {} required for Android builds).", JDK_VERSION);

    if Confirm::new("Install an Orbital-managed Temurin JDK automatically?")
        .with_default(true)
        .prompt()?
    {
        let home = download_java(JDK_VERSION)?;
        println!("Java installed to: {}", home.display());
        Ok(home)
    } else {
        println!("\nPlease install a JDK {} (or newer) manually and set JAVA_HOME.", JDK_VERSION);
        println!("  - https://adoptium.net");
        println!("Then run 'orbital build android' again.");
        anyhow::bail!("Java runtime required but not installed.");
    }
}

/// Downloads and extracts a Temurin JRE into the Orbital cache dir.
/// Returns the JRE home directory.
fn download_java(version: &str) -> Result<PathBuf> {
    let (os, arch, ext) = platform_tokens();

    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{version}/ga/{os}/{arch}/jdk/hotspot/normal/eclipse"
    );

    let dest_dir = crate::tooling::java_dir(version)?;
    std::fs::create_dir_all(&dest_dir).context("Failed to create JDK directory")?;

    println!("\nDownloading Temurin JDK {} ({os}/{arch})...", version);
    println!("  {url}");

    let archive_path = dest_dir.join(format!("jdk.{}", ext));

    // Download with progress bar
    crate::tooling::download_with_progress(&url, &archive_path, "Downloading JDK")?;

    println!("Extracting...");
    if ext == "zip" {
        extract_zip(&archive_path, &dest_dir)?;
    } else {
        extract_tar_gz(&archive_path, &dest_dir)?;
    }

    // Clean up the archive
    std::fs::remove_file(&archive_path).ok();

    // Adoptium zips/tarballs wrap everything in a single top-level dir (e.g. "jdk-25.x+y").
    // Find that dir so we return the actual JDK home.
    let jdk_home = find_jre_home(&dest_dir)?;
    let java = java_executable(&jdk_home);

    if !java.exists() {
        anyhow::bail!("Extracted JDK does not contain a java executable at {}", java.display());
    }
    if !is_jdk(&jdk_home) {
        anyhow::bail!(
            "Extracted JDK is missing javac at {} — Gradle requires a full JDK, not a JRE",
            javac_executable(&jdk_home).display()
        );
    }

    Ok(jdk_home)
}

/// Finds the actual JRE home inside the extraction dir (handles a single wrapping dir).
fn find_jre_home(dest_dir: &Path) -> Result<PathBuf> {
    find_jre_home_in(dest_dir)
}

/// Scans a directory for a nested JDK home.
/// Adoptium archives extract into a single top-level dir (e.g. "jdk-25.x+y").
/// Prefers a full JDK (has `javac`) over a bare JRE.
fn find_jre_home_in(dir: &Path) -> Result<PathBuf> {
    // Fast path: direct bin/java + javac check
    if is_jdk(dir) {
        return Ok(dir.to_path_buf());
    }

    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    entries.sort();

    // Prefer a full JDK
    if let Some(dir) = entries.iter().find(|d| is_jdk(d)) {
        return Ok(dir.clone());
    }

    // Otherwise fall back to any dir that contains java
    if let Some(dir) = entries.iter().find(|d| java_executable(d).exists()) {
        return Ok(dir.clone());
    }

    // Otherwise return the single top-level dir if there's exactly one
    if entries.len() == 1 {
        return Ok(entries[0].clone());
    }

    // No wrapper dir; files went directly into dir
    Ok(dir.to_path_buf())
}

fn platform_tokens() -> (&'static str, &'static str, &'static str) {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x64"
    };

    let ext = if cfg!(windows) { "zip" } else { "tar.gz" };

    (os, arch, ext)
}

fn java_works(java: &Path) -> bool {
    Command::new(java)
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn java_works_on_path() -> bool {
    Command::new("java")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn find_java_on_path() -> Option<PathBuf> {
    if cfg!(windows) {
        // `where java` returns full path
        if let Ok(out) = Command::new("where").arg("java").output() {
            if out.status.success() {
                if let Ok(s) = String::from_utf8(out.stdout) {
                    if let Some(line) = s.lines().next() {
                        let p = PathBuf::from(line.trim());
                        if p.exists() {
                            return Some(p);
                        }
                    }
                }
            }
        }
        None
    } else {
        if let Ok(out) = Command::new("sh").args(["-c", "command -v java"]).output() {
            if out.status.success() {
                if let Ok(s) = String::from_utf8(out.stdout) {
                    let p = PathBuf::from(s.trim());
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
        None
    }
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(archive)
        .map_err(|e| anyhow::anyhow!("Failed to open zip: {}", e))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| anyhow::anyhow!("Failed to read zip: {}", e))?;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)
            .map_err(|e| anyhow::anyhow!("Failed to read zip entry: {}", e))?;
        let outpath = dest.join(entry.mangled_name());

        if entry.is_dir() {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| anyhow::anyhow!("Failed to create dir: {}", e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| anyhow::anyhow!("Failed to create dir: {}", e))?;
                }
            }
            let mut out_file = std::fs::File::create(&outpath)
                .map_err(|e| anyhow::anyhow!("Failed to create file: {}", e))?;
            std::io::copy(&mut entry, &mut out_file)
                .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
        }
    }

    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(archive)
        .map_err(|e| anyhow::anyhow!("Failed to open archive: {}", e))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(gz);

    tar.unpack(dest)
        .map_err(|e| anyhow::anyhow!("Failed to extract archive: {}", e))?;

    Ok(())
}
