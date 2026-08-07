use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};
use std::fs;
use std::path::Path;

use crate::config;

static TEMPLATE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/template");

pub fn init() -> Result<()> {
    let workspace_root = config::find_workspace_root()?;
    let android_dir = workspace_root.join("Android");

    if android_dir.exists() {
        println!("Android/ directory already exists. Skipping generation.");
        return Ok(());
    }

    let android_config = config::load_android_config()?;

    println!("Generating Android project...");
    println!("  Package: {}", android_config.package_name());
    println!("  Min SDK: {}", android_config.min_sdk());
    println!("  Target SDK: {}", android_config.target_sdk());

    generate_project(&android_dir, &android_config)?;

    println!(
        "\nAndroid project generated successfully at {}",
        android_dir.display()
    );
    println!("\nNext steps:");
    println!("  1. Make sure you have the Android SDK and NDK installed");
    println!("  2. Run: orbital build android --example <example_name>");

    Ok(())
}

fn generate_project(android_dir: &Path, config: &config::AndroidConfig) -> Result<()> {
    let replacements = create_replacements(config);

    copy_dir_recursive(&TEMPLATE_DIR, android_dir, &replacements)
        .context("Failed to copy template files")?;

    // Make gradlew executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let gradlew_path = android_dir.join("gradlew");
        if gradlew_path.exists() {
            let mut perms = fs::metadata(&gradlew_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&gradlew_path, perms)?;
        }
    }

    Ok(())
}

fn create_replacements(config: &config::AndroidConfig) -> Vec<(String, String)> {
    vec![
        ("@@@PACKAGE_NAME@@@".to_string(), config.package_name().to_string()),
        ("@@@MIN_SDK@@@".to_string(), config.min_sdk().to_string()),
        ("@@@TARGET_SDK@@@".to_string(), config.target_sdk().to_string()),
        ("@@@AGP_VERSION@@@".to_string(), "9.3.1".to_string()),
        ("@@@NDK_VERSION@@@".to_string(), "26.2.11394342".to_string()),
        // These will be replaced later during build
        ("@@@LIBRARY_NAME@@@".to_string(), "placeholder".to_string()),
        ("@@@APP_NAME@@@".to_string(), "Orbital App".to_string()),
    ]
}

fn copy_dir_recursive(
    src_dir: &Dir,
    dest_dir: &Path,
    replacements: &[(String, String)],
) -> Result<()> {
    fs::create_dir_all(dest_dir)
        .with_context(|| format!("Failed to create directory: {}", dest_dir.display()))?;

    // Copy all files in this directory
    for file in src_dir.files() {
        let file_path = file.path();
        let dest_file = dest_dir.join(file_path);

        if let Some(parent) = dest_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let content = file.contents();

        // Check if this is a text file that needs placeholder replacement
        if is_text_file(file_path) {
            let text_content = std::str::from_utf8(content)
                .with_context(|| format!("Failed to parse file as UTF-8: {}", file_path.display()))?;

            let replaced_content = replace_placeholders(text_content, replacements);
            fs::write(&dest_file, replaced_content)
                .with_context(|| format!("Failed to write file: {}", dest_file.display()))?;
        } else {
            // Binary file, copy as-is
            fs::write(&dest_file, content)
                .with_context(|| format!("Failed to write file: {}", dest_file.display()))?;
        }
    }

    // Recursively copy subdirectories
    for dir in src_dir.dirs() {
        let dir_path = dir.path();
        let dir_name = dir_path.file_name().context("Failed to get directory name")?;
        let dest_subdir = dest_dir.join(dir_name);
        copy_dir_recursive(dir, &dest_subdir, replacements)?;
    }

    Ok(())
}

fn is_text_file(path: &Path) -> bool {
    let text_extensions = [
        "gradle", "properties", "xml", "java", "kt", "txt", "rs", "toml", "json", "md",
    ];

    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| text_extensions.contains(&ext))
        .unwrap_or(false)
}

fn replace_placeholders(content: &str, replacements: &[(String, String)]) -> String {
    let mut result = content.to_string();
    for (placeholder, value) in replacements {
        result = result.replace(placeholder, value);
    }
    result
}
