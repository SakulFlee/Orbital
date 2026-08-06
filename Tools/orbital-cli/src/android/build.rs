use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::config;

pub fn build(package_name: Option<&str>, release: bool) -> Result<()> {
    let project_root = config::find_project_root()?;
    let android_dir = project_root.join("Android");

    if !android_dir.exists() {
        println!("Android/ directory not found. Generating project first...");
        super::project::init()?;
    }

    let android_config = config::load_android_config()?;

    let (package, lib_name) = if let Some(pkg) = package_name {
        // Workspace mode: build specified package
        (pkg.to_string(), config::find_package_lib_name(pkg)?)
    } else {
        // Standalone mode: build current project
        (config::get_package_name()?, config::get_lib_name()?)
    };

    println!("Building for Android...");
    println!("  Package: {}", package);
    println!("  Library: {}", lib_name);
    println!("  Mode: {}", if release { "release" } else { "debug" });

    // Update the Android project with the correct library name and app name
    update_android_project(&android_dir, &package, &lib_name, &android_config)?;

    // Run cargo ndk
    let jni_libs_dir = android_dir.join("app").join("src").join("main").join("jniLibs");

    // Clean previous build artifacts
    if jni_libs_dir.exists() {
        std::fs::remove_dir_all(&jni_libs_dir)
            .context("Failed to clean previous jniLibs")?;
    }

    println!("\nRunning cargo ndk...");

    let mut cargo_ndk_args = vec![
        "-t", "arm64-v8a",
        "-t", "armeabi-v7a",
        "-o", jni_libs_dir.to_str().context("Invalid jniLibs path")?,
        "build",
        "--lib",
        "--package", &package,
    ];

    if release {
        cargo_ndk_args.push("--release");
    }

    // Check if cargo-ndk is installed
    let ndk_check = Command::new("cargo")
        .arg("ndk")
        .arg("--version")
        .output();

    if ndk_check.is_err() || !ndk_check.unwrap().status.success() {
        anyhow::bail!(
            "cargo-ndk is not installed.\n\n\
             Install it with:\n  \
             cargo install cargo-ndk\n\n\
             Also ensure you have the Android NDK installed and ANDROID_NDK_HOME set."
        );
    }

    let status = Command::new("cargo")
        .arg("ndk")
        .args(&cargo_ndk_args)
        .current_dir(&project_root)
        .status()
        .context("Failed to run cargo ndk")?;

    if !status.success() {
        anyhow::bail!("cargo ndk build failed");
    }

    // Run gradlew
    println!("\nRunning gradlew assembleDebug...");

    let gradle_task = if release { "assembleRelease" } else { "assembleDebug" };

    let gradlew = if cfg!(windows) {
        android_dir.join("gradlew.bat")
    } else {
        android_dir.join("gradlew")
    };

    let status = Command::new(&gradlew)
        .arg(gradle_task)
        .current_dir(&android_dir)
        .status()
        .context("Failed to run gradlew")?;

    if !status.success() {
        anyhow::bail!("gradlew build failed");
    }

    let apk_path = if release {
        android_dir
            .join("app")
            .join("build")
            .join("outputs")
            .join("apk")
            .join("release")
    } else {
        android_dir
            .join("app")
            .join("build")
            .join("outputs")
            .join("apk")
            .join("debug")
    };

    println!("\nBuild successful!");
    println!("APK output: {}", apk_path.display());

    Ok(())
}

fn update_android_project(
    android_dir: &Path,
    package_name: &str,
    lib_name: &str,
    config: &config::AndroidConfig,
) -> Result<()> {
    // Update AndroidManifest.xml
    let manifest_path = android_dir
        .join("app")
        .join("src")
        .join("main")
        .join("AndroidManifest.xml");

    if manifest_path.exists() {
        let content = std::fs::read_to_string(&manifest_path)
            .context("Failed to read AndroidManifest.xml")?;
        let content = content.replace("@@@LIBRARY_NAME@@@", lib_name);
        std::fs::write(&manifest_path, content)
            .context("Failed to write AndroidManifest.xml")?;
    }

    // Update strings.xml
    let strings_path = android_dir
        .join("app")
        .join("src")
        .join("main")
        .join("res")
        .join("values")
        .join("strings.xml");

    if strings_path.exists() {
        let content = std::fs::read_to_string(&strings_path)
            .context("Failed to read strings.xml")?;
        let content = content.replace("@@@APP_NAME@@@", package_name);
        std::fs::write(&strings_path, content)
            .context("Failed to write strings.xml")?;
    }

    // Update app/build.gradle
    let build_gradle_path = android_dir.join("app").join("build.gradle");
    if build_gradle_path.exists() {
        let content = std::fs::read_to_string(&build_gradle_path)
            .context("Failed to read app/build.gradle")?;
        let content = content.replace("@@@PACKAGE_NAME@@@", &format!("{}.{}", config.package_name(), package_name));
        let content = content.replace("@@@MIN_SDK@@@", &config.min_sdk().to_string());
        let content = content.replace("@@@TARGET_SDK@@@", &config.target_sdk().to_string());
        std::fs::write(&build_gradle_path, content)
            .context("Failed to write app/build.gradle")?;
    }

    Ok(())
}
