use anyhow::{Context, Result};
use std::process::Command;

/// Builds the desktop binary of the current project.
pub fn build(release: bool) -> Result<()> {
    let project_root = crate::config::find_project_root()?;

    // Sync engine assets so runtime path loads (e.g. "Shaders/pbr.wgsl")
    // resolve against <cwd>/Assets on desktop.
    crate::assets::sync_assets(&project_root)?;

    println!("Building for Desktop...");
    println!("  Mode: {}", if release { "release" } else { "debug" });

    let mut args = vec!["build"];
    if release {
        args.push("--release");
    }

    let status = Command::new("cargo")
        .args(&args)
        .current_dir(&project_root)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    println!("\nBuild successful!");
    Ok(())
}

/// Runs the desktop binary of the current project.
pub fn run() -> Result<()> {
    let project_root = crate::config::find_project_root()?;

    // Sync engine assets so runtime path loads resolve on desktop.
    crate::assets::sync_assets(&project_root)?;

    println!("Running Desktop app...");

    let status = Command::new("cargo")
        .arg("run")
        .current_dir(&project_root)
        .status()
        .context("Failed to run cargo run")?;

    if !status.success() {
        anyhow::bail!("cargo run failed");
    }

    Ok(())
}
