use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::config;

pub fn run(package_name: Option<&str>, device_id: Option<&str>, skip_build: bool) -> Result<()> {
    // Check that we're on macOS
    if cfg!(not(target_os = "macos")) {
        anyhow::bail!(
            "iOS deployment requires macOS with Xcode installed.\n\
             This command can only be run on a Mac."
        );
    }

    // First build (unless skipped)
    if !skip_build {
        super::build::build(package_name, false)?;
    }

    let ios_config = config::load_ios_config()?;
    let bundle_id = ios_config.bundle_id();

    // Select and boot simulator
    let sim = super::device::select_simulator(device_id)?;
    super::device::boot_simulator(&sim.udid)?;

    // Find the .app bundle
    let project_root = config::find_project_root()?;
    let app_name = ios_config.app_name();

    // xcodebuild output is typically in DerivedData, but we can also look
    // in the build directory
    let build_dir = project_root
        .join("Build")
        .join("Products")
        .join("Debug-iphonesimulator")
        .join(format!("{}.app", app_name));

    // Also check the derived data path
    let derived_data = std::env::var("HOME")
        .map(|home| {
            Path::new(&home)
                .join("Library")
                .join("Developer")
                .join("Xcode")
                .join("DerivedData")
        })
        .ok();

    let app_path = if build_dir.exists() {
        build_dir
    } else if let Some(dd) = derived_data {
        // Search DerivedData for our .app
        find_app_in_derived_data(&dd, app_name)?.unwrap_or_else(|| {
            println!("Warning: Could not find .app bundle. Using default path.");
            build_dir
        })
    } else {
        build_dir
    };

    if !app_path.exists() {
        anyhow::bail!(
            "App bundle not found at: {}\n\
             Run 'orbital build ios' first.",
            app_path.display()
        );
    }

    // Install on simulator
    println!("Installing on simulator '{}'...", sim.name);
    let status = Command::new("xcrun")
        .args(["simctl", "install", &sim.udid])
        .arg(&app_path)
        .status()
        .context("Failed to install app on simulator")?;

    if !status.success() {
        anyhow::bail!("Failed to install app on simulator");
    }

    // Launch the app
    println!("Launching app...");
    let output = Command::new("xcrun")
        .args(["simctl", "launch", &sim.udid, bundle_id])
        .output()
        .context("Failed to launch app on simulator")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("Already running") {
            anyhow::bail!("Failed to launch app: {}", stderr.trim());
        }
    }

    println!("\nApp launched successfully on '{}'!", sim.name);

    // Stream logs
    println!("Streaming logs (Ctrl+C to stop)...\n");
    let _ = Command::new("xcrun")
        .args([
            "simctl",
            "spawn",
            &sim.udid,
            "log",
            "stream",
            "--level",
            "debug",
            "--predicate",
            &format!("subsystem == \"com.apple.UnityFramework\" OR (process == \"{}\" AND eventMessage CONTAINS[c] \"rust\")", bundle_id),
        ])
        .status();

    Ok(())
}

/// Search DerivedData for an .app bundle matching the given name.
fn find_app_in_derived_data(
    derived_data: &Path,
    app_name: &str,
) -> Result<Option<std::path::PathBuf>> {
    let suffix = format!("{}.app", app_name);

    if !derived_data.exists() {
        return Ok(None);
    }

    for entry in std::fs::read_dir(derived_data)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        // Check Build/Products/Debug-iphonesimulator/
        let candidate = entry
            .path()
            .join("Build")
            .join("Products")
            .join("Debug-iphonesimulator")
            .join(&suffix);

        if candidate.exists() {
            return Ok(Some(candidate));
        }

        // Also check Release-iphonesimulator
        let candidate = entry
            .path()
            .join("Build")
            .join("Products")
            .join("Release-iphonesimulator")
            .join(&suffix);

        if candidate.exists() {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}
