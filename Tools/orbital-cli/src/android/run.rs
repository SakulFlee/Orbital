use anyhow::{Context, Result};
use std::process::Command;

use crate::config;

pub fn run(package_name: Option<&str>) -> Result<()> {
    // First build
    super::build::build(package_name, false)?;

    println!("\nInstalling on connected device...");

    // Find the APK
    let project_root = config::find_project_root()?;
    let apk_path = project_root
        .join("Android")
        .join("app")
        .join("build")
        .join("outputs")
        .join("apk")
        .join("debug")
        .join("app-debug.apk");

    if !apk_path.exists() {
        anyhow::bail!("APK not found at: {}", apk_path.display());
    }

    // Install via adb
    let status = Command::new("adb")
        .arg("install")
        .arg("-r")
        .arg(&apk_path)
        .status()
        .context("Failed to run adb install. Is adb in your PATH?")?;

    if !status.success() {
        anyhow::bail!("adb install failed");
    }

    // Get package name for launch
    let android_config = config::load_android_config()?;
    let package = if let Some(pkg) = package_name {
        pkg.to_string()
    } else {
        config::get_package_name()?
    };
    let full_package = format!("{}.{}", android_config.package_name(), package);

    println!("Launching app...");

    // Launch the app
    let status = Command::new("adb")
        .arg("shell")
        .arg("am")
        .arg("start")
        .arg("-n")
        .arg(format!("{}/android.app.NativeActivity", full_package))
        .status()
        .context("Failed to run adb shell am start")?;

    if !status.success() {
        anyhow::bail!("Failed to launch app");
    }

    println!("\nApp launched successfully!");
    println!("To view logs: adb logcat -s rust_std_out");

    Ok(())
}
