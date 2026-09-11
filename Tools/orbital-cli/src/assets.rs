use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Locates the engine's `Assets/` directory for the project at `project_root`.
///
/// The engine is pulled in as a git dependency, so we resolve the exact
/// revision the project compiles against via `cargo metadata` and return the
/// `Assets/` directory at the root of that checkout. Returns `None` (without
/// error) when the engine package or its `Assets/` cannot be located.
pub fn engine_assets_dir(project_root: &Path) -> Result<Option<PathBuf>> {
    let output = Command::new("cargo")
        .arg("metadata")
        .args(["--format-version", "1"])
        .current_dir(project_root)
        .output()
        .context("Failed to run cargo metadata")?;

    if !output.status.success() {
        return Ok(None);
    }

    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse cargo metadata")?;

    let packages = metadata
        .get("packages")
        .and_then(|p| p.as_array())
        .context("cargo metadata missing packages")?;

    for package in packages {
        let name = package.get("name").and_then(|n| n.as_str()).unwrap_or("");
        if name != "orbital" {
            continue;
        }
        let manifest_path = package
            .get("manifest_path")
            .and_then(|m| m.as_str())
            .context("orbital package missing manifest_path")?;
        let manifest = Path::new(manifest_path);
        // <repo>/Crates/orbital/Cargo.toml -> <repo>
        let repo_root = manifest
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent());
        if let Some(repo_root) = repo_root {
            let assets = repo_root.join("Assets");
            if assets.is_dir() {
                return Ok(Some(assets));
            }
        }
        return Ok(None);
    }

    Ok(None)
}

/// Syncs the engine's `Assets/` directory into the project's `Assets/`.
///
/// Files from the engine are copied over the destination (newer engine
/// revisions win). Files present in the destination but not in the engine are
/// left untouched so user-provided assets are preserved.
pub fn sync_assets(project_root: &Path) -> Result<()> {
    let Some(engine_assets) = engine_assets_dir(project_root)? else {
        println!("Engine Assets/ directory not found; skipping asset sync.");
        return Ok(());
    };

    let dest = project_root.join("Assets");
    if !dest.exists() {
        std::fs::create_dir_all(&dest)
            .with_context(|| format!("Failed to create {}", dest.display()))?;
    }

    copy_tree(&engine_assets, &dest, &engine_assets)?;
    println!(
        "Synced engine assets from {} to {}",
        engine_assets.display(),
        dest.display()
    );
    Ok(())
}

fn copy_tree(src_root: &Path, dst_root: &Path, src_dir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(src_dir)
        .with_context(|| format!("Failed to read dir {}", src_dir.display()))?
    {
        let entry = entry.context("Failed to read directory entry")?;
        let src_path = entry.path();
        let rel = src_path
            .strip_prefix(src_root)
            .context("Failed to strip source prefix")?;
        let dst_path = dst_root.join(rel);

        let file_type = entry
            .file_type()
            .with_context(|| format!("Failed to get file type for {}", src_path.display()))?;

        if file_type.is_dir() {
            std::fs::create_dir_all(&dst_path)
                .with_context(|| format!("Failed to create {}", dst_path.display()))?;
            copy_tree(src_root, dst_root, &src_path)?;
        } else if file_type.is_file() {
            std::fs::copy(&src_path, &dst_path).with_context(|| {
                format!(
                    "Failed to copy {} -> {}",
                    src_path.display(),
                    dst_path.display()
                )
            })?;
        }
    }
    Ok(())
}
