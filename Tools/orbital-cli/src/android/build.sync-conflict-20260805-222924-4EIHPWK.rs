use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::config;

pub fn build(example_name: &str, release: bool) -> Result<()> {
    let workspace_root = config::find_workspace_root()?;
    let android_dir = workspace_root.join("Android");

    if !android_dir.exists() {
        println!("Android/ directory not found. Generating project first...");
        super::project::init()?;
    }

    let android_config = config::load_android_config()?;
    let lib_name = config::find_example_lib_name(example_name)?;
    let example_path = config::find_example_path(example_name)?;

    println!("Building for Android...");
    println!("  Example: {}", example_name);
    println!("  Library: {}", lib_name);
    println!("  Mode: {}", if release { "release" } else { "debug" });

    // Update the Android project with the correct library name and app name
    update_android_project(&android_dir, example_name, &lib_name, &android_config)?;

    // Run cargo ndk
    let build_mode = if release { "--release" } else { "" };
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
        "--package", example_name,
    ];

    if release {
        cargo_ndk_args.push("--release");
    }

    let status = Command::new("cargo")
        .arg("ndk")
        .args(&cargo_ndk_args)
        .current_dir(&workspace_root)
        .status()
        .context("Failed to run cargo ndk. Is cargo-ndk installed?")?;

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
    example_name: &str,
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
        let content = content.replace("@@@APP_NAME@@@", example_name);
        std::fs::write(&strings_path, content)
            .context("Failed to write strings.xml")?;
    }

    // Update app/build.gradle
    let build_gradle_path = android_dir.join("app").join("build.gradle");
    if build_gradle_path.exists() {
        let content = std::fs::read_to_string(&build_gradle_path)
            .context("Failed to read app/build.gradle")?;
        let content = content.replace("@@@PACKAGE_NAME@@@", &format!("{}.{}", config.package_name(), example_name));
        let content = content.replace("@@@MIN_SDK@@@", &config.min_sdk().to_string());
        let content = content.replace("@@@TARGET_SDK@@@", &config.target_sdk().to_string());
        std::fs::write(&build_gradle_path, content)
            .context("Failed to write app/build.gradle")?;
    }

    Ok(())
}
