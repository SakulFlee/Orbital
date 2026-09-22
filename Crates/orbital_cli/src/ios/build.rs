use anyhow::{Context, Result};
use std::process::Command;

use crate::config;

pub fn build(package_name: Option<&str>, release: bool) -> Result<()> {
    // Check that we're on macOS (xcodebuild is macOS-only)
    if cfg!(not(target_os = "macos")) {
        anyhow::bail!(
            "iOS builds require macOS with Xcode installed.\n\
             This command can only be run on a Mac."
        );
    }

    let project_root = config::find_project_root()?;
    let ios_dir = project_root.join("iOS");

    if !ios_dir.exists() {
        println!("iOS/ directory not found. Generating project first...");
        super::project::init()?;
    }

    let ios_config = config::load_ios_config()?;

    let lib_name = if let Some(pkg) = package_name {
        config::find_package_lib_name(pkg)?
    } else {
        config::get_lib_name()?
    };

    let package = package_name.unwrap_or(&lib_name);

    println!("Building for iOS...");
    println!("  Package: {}", package);
    println!("  Library: {}", lib_name);
    println!("  Mode: {}", if release { "release" } else { "debug" });

    // Ensure aarch64-apple-ios target is installed
    ensure_ios_target()?;

    // Build the Rust static library
    println!("\nBuilding Rust static library...");
    let mut cargo_args = vec!["build", "--lib", "--target", "aarch64-apple-ios"];
    if let Some(pkg) = package_name {
        cargo_args.push("--package");
        cargo_args.push(pkg);
    }
    if release {
        cargo_args.push("--release");
    }

    let status = Command::new("cargo")
        .args(&cargo_args)
        .current_dir(&project_root)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    // Build with xcodebuild
    println!("\nBuilding Xcode project...");

    let configuration = if release { "Release" } else { "Debug" };
    let scheme = &ios_config.app_name().replace(' ', "_");

    let status = Command::new("xcodebuild")
        .args([
            "-project",
            &ios_dir
                .join(format!("{}.xcodeproj", ios_config.app_name()))
                .to_string_lossy(),
            "-scheme",
            scheme,
            "-sdk",
            "iphonesimulator",
            "-configuration",
            configuration,
            "build",
        ])
        .current_dir(&project_root)
        .status()
        .context("Failed to run xcodebuild")?;

    if !status.success() {
        anyhow::bail!("xcodebuild failed");
    }

    println!("\nBuild successful!");

    Ok(())
}

/// Ensure the aarch64-apple-ios target is installed.
fn ensure_ios_target() -> Result<()> {
    let target = "aarch64-apple-ios";

    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .context("Failed to run 'rustup target list'")?;

    let installed = String::from_utf8_lossy(&output.stdout);
    if installed.contains(target) {
        return Ok(());
    }

    println!("Installing iOS target {}...", target);
    let status = Command::new("rustup")
        .args(["target", "install", target])
        .status()
        .context("Failed to install iOS target")?;

    if !status.success() {
        anyhow::bail!("Failed to install target: {}", target);
    }

    Ok(())
}
