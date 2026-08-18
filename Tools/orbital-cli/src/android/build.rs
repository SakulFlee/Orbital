use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::Write;
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

    // Ensure Android SDK is configured
    super::sdk::ensure_android_sdk()?;

    // Get SDK and NDK paths for cargo-ndk
    let sdk_path =
        super::sdk::current_sdk_path().context("Android SDK path not available after setup")?;
    let ndk_version = android_config.ndk_version();
    let ndk_path = super::sdk::ndk_path(&sdk_path, ndk_version);

    // Verify NDK exists
    if !ndk_path.exists() {
        anyhow::bail!(
            "Android NDK not found at: {}\n\
             Run 'orbital build android' again to install it, or set ANDROID_NDK_HOME.",
            ndk_path.display()
        );
    }

    // Ensure Java is available (gradlew needs it)
    let java_home = crate::java::ensure_java()?;

    // Update the Android project with the correct library name and app name
    update_android_project(&android_dir, &package, &lib_name, &android_config)?;

    // Ensure cargo-ndk is installed
    ensure_cargo_ndk()?;

    // Ensure required Rust targets are installed
    let targets = android_config.targets();
    ensure_rust_targets(&targets)?;

    // Run cargo ndk
    let jni_libs_dir = android_dir
        .join("app")
        .join("src")
        .join("main")
        .join("jniLibs");

    // Clean previous build artifacts
    if jni_libs_dir.exists() {
        std::fs::remove_dir_all(&jni_libs_dir).context("Failed to clean previous jniLibs")?;
    }

    println!("\nRunning cargo ndk...");

    // Use to_string_lossy to handle non-UTF-8 paths on Unix
    let jni_libs_str = jni_libs_dir.to_string_lossy().to_string();
    let mut cargo_ndk_args = vec!["-o", &jni_libs_str, "build", "--lib", "--package", &package];

    // Add targets from config
    for target in &targets {
        cargo_ndk_args.push("-t");
        cargo_ndk_args.push(target);
    }

    if release {
        cargo_ndk_args.push("--release");
    }

    let status = Command::new("cargo")
        .arg("ndk")
        .args(&cargo_ndk_args)
        .env("ANDROID_NDK_HOME", &ndk_path)
        .current_dir(&project_root)
        .status()
        .context("Failed to run cargo ndk")?;

    if !status.success() {
        anyhow::bail!("cargo ndk build failed");
    }

    // Run gradlew
    let apk_mode = android_config.apk_mode();
    println!("\nRunning gradlew...");

    let gradlew = if cfg!(windows) {
        android_dir.join("gradlew.bat")
    } else {
        android_dir.join("gradlew")
    };

    match apk_mode {
        "single" => {
            // Build separate APKs for each architecture
            for target in &targets {
                let abi_dir = jni_libs_dir.join(target);
                if !abi_dir.exists() {
                    continue;
                }

                println!("\nBuilding APK for {}...", target);

                // Clean build for this ABI
                let build_dir = android_dir.join("app").join("build");
                if build_dir.exists() {
                    std::fs::remove_dir_all(&build_dir).ok();
                }

                // Build with split ABI
                let gradle_task = if release {
                    "assembleRelease"
                } else {
                    "assembleDebug"
                };
                let status = Command::new(&gradlew)
                    .env("JAVA_HOME", &java_home)
                    .args([gradle_task, &format!("-PtargetAbi={}", target)])
                    .current_dir(&android_dir)
                    .status()
                    .context("Failed to run gradlew")?;

                if !status.success() {
                    anyhow::bail!("gradlew build failed for {}", target);
                }
            }
        }
        "both" => {
            // Build multiarch first, then single APKs
            println!("\nBuilding multiarch APK...");
            let gradle_task = if release {
                "assembleRelease"
            } else {
                "assembleDebug"
            };
            let status = Command::new(&gradlew)
                .env("JAVA_HOME", &java_home)
                .arg(gradle_task)
                .current_dir(&android_dir)
                .status()
                .context("Failed to run gradlew")?;

            if !status.success() {
                anyhow::bail!("gradlew multiarch build failed");
            }

            // Then build single APKs
            for target in &targets {
                let abi_dir = jni_libs_dir.join(target);
                if !abi_dir.exists() {
                    continue;
                }

                println!("\nBuilding single APK for {}...", target);

                let build_dir = android_dir.join("app").join("build");
                if build_dir.exists() {
                    std::fs::remove_dir_all(&build_dir).ok();
                }

                let status = Command::new(&gradlew)
                    .env("JAVA_HOME", &java_home)
                    .args([gradle_task, &format!("-PtargetAbi={}", target)])
                    .current_dir(&android_dir)
                    .status()
                    .context("Failed to run gradlew")?;

                if !status.success() {
                    anyhow::bail!("gradlew build failed for {}", target);
                }
            }
        }
        _ => {
            // Default: multiarch (all targets in one APK)
            let gradle_task = if release {
                "assembleRelease"
            } else {
                "assembleDebug"
            };
            let status = Command::new(&gradlew)
                .env("JAVA_HOME", &java_home)
                .arg(gradle_task)
                .current_dir(&android_dir)
                .status()
                .context("Failed to run gradlew")?;

            if !status.success() {
                anyhow::bail!("gradlew build failed");
            }
        }
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

/// Replaces the value of an existing `android:screenOrientation` attribute so
/// orientation changes from the config take effect on rebuild. Returns the
/// content unchanged if the attribute is not present.
fn set_screen_orientation_value(content: &str, screen_orientation: &str) -> String {
    const MARKER: &str = "android:screenOrientation=\"";
    match content.find(MARKER) {
        Some(start) => {
            let value_start = start + MARKER.len();
            match content[value_start..].find('"') {
                Some(len) => {
                    let value_end = value_start + len;
                    format!(
                        "{}{}{}",
                        &content[..value_start],
                        screen_orientation,
                        &content[value_end..]
                    )
                }
                None => content.to_string(),
            }
        }
        None => content.to_string(),
    }
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
        // Migrate projects generated by older CLI versions that baked in a
        // literal "placeholder" value.
        let content = content.replace("placeholder", lib_name);

        // Resolve the screen orientation. New templates carry the
        // @@@SCREEN_ORIENTATION@@@ placeholder, which is resolved on first
        // build. On later rebuilds the attribute already holds a literal
        // value, which is rewritten from the config so that changes to
        // Orbital.toml take effect. Manifests generated before this feature
        // existed (which never had the attribute or placeholder) get the
        // attribute inserted before the configChanges line.
        let screen_orientation = config.screen_orientation();
        let content = content.replace("@@@SCREEN_ORIENTATION@@@", screen_orientation);
        let content = if content.contains("android:screenOrientation=") {
            set_screen_orientation_value(&content, screen_orientation)
        } else {
            let anchor = "android:configChanges=\"orientation|keyboardHidden|screenSize\"";
            let replacement =
                format!("android:screenOrientation=\"{screen_orientation}\"\n            {anchor}");
            match content.replacen(anchor, &replacement, 1) {
                updated if updated != content => updated,
                _ => {
                    anyhow::bail!(
                        "Failed to insert android:screenOrientation into {}: \
                         expected the activity to contain {anchor}",
                        manifest_path.display()
                    )
                }
            }
        };

        std::fs::write(&manifest_path, content).context("Failed to write AndroidManifest.xml")?;
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
        let content =
            std::fs::read_to_string(&strings_path).context("Failed to read strings.xml")?;
        let content = content.replace("@@@APP_NAME@@@", package_name);
        std::fs::write(&strings_path, content).context("Failed to write strings.xml")?;
    }

    // Update app/build.gradle
    let build_gradle_path = android_dir.join("app").join("build.gradle");
    if build_gradle_path.exists() {
        let content = std::fs::read_to_string(&build_gradle_path)
            .context("Failed to read app/build.gradle")?;
        let content = content.replace("@@@PACKAGE_NAME@@@", config.package_name());
        let content = content.replace("@@@MIN_SDK@@@", &config.min_sdk().to_string());
        let content = content.replace("@@@TARGET_SDK@@@", &config.target_sdk().to_string());
        std::fs::write(&build_gradle_path, content).context("Failed to write app/build.gradle")?;
    }

    Ok(())
}

/// Ensure cargo-ndk is installed, auto-install if missing
fn ensure_cargo_ndk() -> Result<()> {
    // Check if cargo-ndk is already installed
    let ndk_check = Command::new("cargo").arg("ndk").arg("--version").output();

    if let Ok(output) = ndk_check
        && output.status.success()
    {
        return Ok(());
    }

    // cargo-ndk not found, attempt to install it
    println!("cargo-ndk not found. Installing...");

    let status = Command::new("cargo")
        .args(["install", "cargo-ndk"])
        .status()
        .context("Failed to run 'cargo install cargo-ndk'. Is Cargo installed?")?;

    if !status.success() {
        anyhow::bail!(
            "Failed to install cargo-ndk automatically.\n\n\
             Please install it manually:\n  \
             cargo install cargo-ndk\n\n\
             Also ensure you have the Android NDK installed and ANDROID_NDK_HOME set."
        );
    }

    println!("cargo-ndk installed successfully!");

    // Verify installation
    let verify = Command::new("cargo")
        .arg("ndk")
        .arg("--version")
        .output()
        .context("Failed to verify cargo-ndk installation")?;

    if !verify.status.success() {
        anyhow::bail!(
            "cargo-ndk installation completed but verification failed.\n\
             Please restart your terminal and try again."
        );
    }

    Ok(())
}

/// Ensure required Rust targets are installed, prompt user to install if missing
fn ensure_rust_targets(targets: &[String]) -> Result<()> {
    let target_map = HashMap::from([
        ("arm64-v8a", "aarch64-linux-android"),
        ("armeabi-v7a", "armv7-linux-androideabi"),
        ("x86_64", "x86_64-linux-android"),
        ("x86", "i686-linux-android"),
    ]);

    // Get installed targets
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .context("Failed to run 'rustup target list'. Is rustup installed?")?;

    let installed = String::from_utf8_lossy(&output.stdout);

    // Check which targets are missing
    let mut missing = Vec::new();
    for abi in targets {
        if let Some(triple) = target_map.get(abi.as_str())
            && !installed.contains(triple)
        {
            missing.push(triple.to_string());
        }
    }

    if missing.is_empty() {
        return Ok(());
    }

    // Ask user if they want to install missing targets
    println!("\nMissing Rust targets: {}", missing.join(", "));
    print!("Install them now? [Y/n] ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "n" {
        anyhow::bail!(
            "Cannot build without required targets.\n\
             Install them with:\n  \
             rustup target install {}",
            missing.join(" ")
        );
    }

    for target in &missing {
        println!("Installing {}...", target);
        let status = Command::new("rustup")
            .args(["target", "install", target])
            .status()
            .context("Failed to run 'rustup target install'")?;

        if !status.success() {
            anyhow::bail!("Failed to install target: {}", target);
        }
    }

    println!("All targets installed successfully!");

    Ok(())
}
