use anyhow::{Context, Result};
use inquire::{Confirm, Text};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Check if Android SDK and NDK are properly configured
pub fn ensure_android_sdk() -> Result<()> {
    // Check if ANDROID_HOME or ANDROID_SDK_ROOT is set
    let sdk_path = std::env::var("ANDROID_HOME")
        .or_else(|_| std::env::var("ANDROID_SDK_ROOT"));

    match sdk_path {
        Ok(path) => {
            let sdk_path = PathBuf::from(path);
            if sdk_path.exists() {
                println!("Android SDK found at: {}", sdk_path.display());
                return Ok(());
            }
            println!("ANDROID_HOME is set but path doesn't exist: {}", sdk_path.display());
        }
        Err(_) => {
            println!("Android SDK not found (ANDROID_HOME not set).");
        }
    }

    // Ask user for SDK path or offer to download
    println!("\nAndroid SDK is required for building Android apps.");
    println!("Options:");
    println!("  1. Provide path to existing Android SDK");
    println!("  2. Download Android command-line tools");
    println!("  3. Skip (you'll need to set up SDK manually)");

    let choice = Text::new("Choose an option [1/2/3]:")
        .with_default("1")
        .prompt()?;

    match choice.as_str() {
        "1" => {
            // Ask for SDK path
            let sdk_path = Text::new("Enter Android SDK path:")
                .prompt()?;

            let sdk_path = PathBuf::from(&sdk_path);
            if !sdk_path.exists() {
                anyhow::bail!("Path does not exist: {}", sdk_path.display());
            }

            // Set ANDROID_HOME for this session
            // SAFETY: We're setting an environment variable for the current process
            unsafe { std::env::set_var("ANDROID_HOME", &sdk_path) };
            println!("ANDROID_HOME set to: {}", sdk_path.display());

            // Check for sdkmanager
            ensure_sdkmanager(&sdk_path)?;
        }
        "2" => {
            // Download Android command-line tools
            download_sdk()?;
        }
        "3" => {
            println!("Skipping SDK setup. Make sure to set ANDROID_HOME before building.");
        }
        _ => {
            println!("Invalid option. Skipping SDK setup.");
        }
    }

    Ok(())
}

/// Ensure sdkmanager is available and NDK is installed
fn ensure_sdkmanager(sdk_path: &PathBuf) -> Result<()> {
    let sdkmanager = if cfg!(windows) {
        sdk_path.join("cmdline-tools").join("latest").join("bin").join("sdkmanager.bat")
    } else {
        sdk_path.join("cmdline-tools").join("latest").join("bin").join("sdkmanager")
    };

    if !sdkmanager.exists() {
        println!("\nsdkmanager not found at: {}", sdkmanager.display());
        println!("You may need to install Android command-line tools.");

        if Confirm::new("Install Android command-line tools now?")
            .with_default(true)
            .prompt()?
        {
            download_sdk()?;
        } else {
            println!("Please install Android SDK manually.");
        }
        return Ok(());
    }

    // Check if NDK is installed
    let ndk_installed = Command::new(&sdkmanager)
        .args(["--list_installed"])
        .output();

    if let Ok(output) = ndk_installed {
        let output_str = String::from_utf8_lossy(&output.stdout);
        if !output_str.contains("ndk;") {
            println!("\nAndroid NDK not installed.");

            if Confirm::new("Install Android NDK now?")
                .with_default(true)
                .prompt()?
            {
                // Find latest NDK version
                let list_output = Command::new(&sdkmanager)
                    .args(["--list"])
                    .output();

                if let Ok(list) = list_output {
                    let list_str = String::from_utf8_lossy(&list.stdout);
                    // Find NDK versions (look for lines starting with "ndk;")
                    let ndk_versions: Vec<&str> = list_str.lines()
                        .filter(|line| line.trim().starts_with("ndk;"))
                        .collect();

                    if let Some(latest_ndk) = ndk_versions.last() {
                        let ndk_version = latest_ndk.split_whitespace().next().unwrap_or("ndk;26.2.11394342");
                        println!("Installing {}...", ndk_version);

                        let status = Command::new(&sdkmanager)
                            .args([ndk_version])
                            .status()
                            .context("Failed to run sdkmanager")?;

                        if status.success() {
                            println!("NDK installed successfully!");
                        } else {
                            anyhow::bail!("Failed to install NDK");
                        }
                    } else {
                        println!("Could not find NDK version. Please install manually:");
                        println!("  {} \"ndk;26.2.11394342\"", sdkmanager.display());
                    }
                }
            } else {
                println!("Please install NDK manually:");
                println!("  {} \"ndk;26.2.11394342\"", sdkmanager.display());
            }
        } else {
            println!("Android NDK is installed.");
        }
    }

    Ok(())
}

/// Fetch the latest command-line tools URL from Google's repository index
fn fetch_latest_commandline_tools_url() -> Result<(String, String)> {
    // Determine the platform suffix we're looking for
    let platform_suffix = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        "mac_arm64"
    } else if cfg!(target_os = "macos") {
        "mac_x86_64"
    } else if cfg!(target_os = "windows") {
        "win"
    } else {
        anyhow::bail!("Unsupported platform");
    };

    // Try to fetch the latest version from Google's repository using ureq
    // If it fails, fall back to a known working version
    let repo_url = "https://dl.google.com/android/repository/repository2-3.xml";

    println!("Fetching latest version from Google's repository...");

    let xml = match ureq::get(repo_url).call() {
        Ok(mut response) => {
            match response.body_mut().read_to_string() {
                Ok(xml) => xml,
                Err(_) => {
                    println!("Using fallback version...");
                    let fallback_url = format!(
                        "https://dl.google.com/android/repository/commandlinetools-{}-15859902_latest.zip",
                        platform_suffix
                    );
                    return Ok((fallback_url, "commandlinetools.zip".to_string()));
                }
            }
        }
        Err(_) => {
            println!("Using fallback version...");
            let fallback_url = format!(
                "https://dl.google.com/android/repository/commandlinetools-{}-15859902_latest.zip",
                platform_suffix
            );
            return Ok((fallback_url, "commandlinetools.zip".to_string()));
        }
    };

    // Simple pattern matching: find URL containing commandlinetools-{platform}-
    let pattern = format!("commandlinetools-{}-", platform_suffix);
    for line in xml.lines() {
        let line = line.trim();
        if line.contains("<url>") && line.contains(&pattern) {
            if let Some(url_start) = line.find("<url>") {
                if let Some(url_end) = line.find("</url>") {
                    let url = &line[url_start + 5..url_end];
                    let full_url = format!("https://dl.google.com/android/repository/{}", url);

                    // Extract version
                    if let Some(pos) = url.find(&pattern) {
                        let after_platform = &url[pos + pattern.len()..];
                        if let Some(dash_pos) = after_platform.find('-') {
                            let version_str = &after_platform[..dash_pos];
                            if let Ok(version) = version_str.parse::<u64>() {
                                println!("Found latest version: {}", version);
                                return Ok((full_url, "commandlinetools.zip".to_string()));
                            }
                        }
                    }

                    // If we couldn't extract version, still return the URL
                    println!("Found latest command-line tools");
                    return Ok((full_url, "commandlinetools.zip".to_string()));
                }
            }
        }
    }

    // Fallback to a known working version
    println!("Using fallback version...");
    let fallback_url = format!(
        "https://dl.google.com/android/repository/commandlinetools-{}-15859902_latest.zip",
        platform_suffix
    );
    Ok((fallback_url, "commandlinetools.zip".to_string()))
}

/// Download Android SDK command-line tools
fn download_sdk() -> Result<()> {
    println!("\nDownloading Android command-line tools...");

    // Fetch the latest version from Google's repository index
    let (url, filename) = fetch_latest_commandline_tools_url()?;

    // Ask user where to install
    let install_path = Text::new("Enter installation path:")
        .with_default(&dirs::home_dir()
            .map(|h| h.join("android-sdk").to_string_lossy().to_string())
            .unwrap_or_else(|| "./android-sdk".to_string()))
        .prompt()?;

    let install_path = PathBuf::from(&install_path);
    std::fs::create_dir_all(&install_path)?;

    // Download the file using ureq (Rust HTTP library)
    println!("Downloading from: {}", url);

    let zip_path = install_path.join(&filename);
    let temp_path = install_path.join(format!("{}.tmp", filename));

    // Remove any existing files
    if zip_path.exists() {
        std::fs::remove_file(&zip_path).ok();
    }
    if temp_path.exists() {
        std::fs::remove_file(&temp_path).ok();
    }

    // Download to temporary file first, then rename
    // This ensures the file is fully written before we try to extract it
    let download_result = (|| -> Result<()> {
        let mut response = ureq::get(&url)
            .call()
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

        let mut file = std::fs::File::create(&temp_path)
            .map_err(|e| anyhow::anyhow!("Failed to create temp file: {}", e))?;

        let mut reader = response.body_mut().as_reader();
        std::io::copy(&mut reader, &mut file)
            .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;

        // Explicitly drop the file handle before renaming
        drop(file);

        // Rename temp file to final name
        std::fs::rename(&temp_path, &zip_path)
            .map_err(|e| anyhow::anyhow!("Failed to rename file: {}", e))?;

        Ok(())
    })();

    // Clean up temp file if it exists
    if temp_path.exists() {
        std::fs::remove_file(&temp_path).ok();
    }

    match download_result {
        Ok(()) => {
            println!("Download complete!");
        }
        Err(e) => {
            println!("\nDownload failed: {}", e);
            show_manual_instructions(&install_path);
            return Ok(());
        }
    }

    // Verify the file was downloaded
    if !zip_path.exists() {
        println!("\nDownloaded file not found.");
        show_manual_instructions(&install_path);
        return Ok(());
    }

    // Extract the file using Rust zip library (avoids file locking issues)
    println!("Extracting...");

    match extract_zip(&zip_path, &install_path) {
        Ok(()) => {
            println!("Extraction complete!");
        }
        Err(e) => {
            println!("\nFailed to extract SDK tools: {}", e);
            show_manual_instructions(&install_path);
            return Ok(());
        }
    }

    // Move cmdline-tools to the right place (with retry for Windows file locking)
    let cmdline_tools = install_path.join("cmdline-tools");
    let latest = install_path.join("cmdline-tools").join("latest");

    if cmdline_tools.exists() && !latest.exists() {
        for attempt in 1..=5 {
            match std::fs::rename(&cmdline_tools, &latest) {
                Ok(()) => break,
                Err(_e) if attempt < 5 => {
                    println!("Waiting for files to be released (attempt {}/5)...", attempt);
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
                Err(e) => {
                    println!("Warning: Could not reorganize SDK directory: {}", e);
                    println!("You may need to manually move cmdline-tools/ to cmdline-tools/latest/");
                }
            }
        }
    }

    // Clean up zip file
    std::fs::remove_file(&zip_path).ok();

    // Set ANDROID_HOME
    // SAFETY: We're setting an environment variable for the current process
    unsafe { std::env::set_var("ANDROID_HOME", &install_path) };
    println!("\nSDK installed to: {}", install_path.display());
    println!("ANDROID_HOME set to: {}", install_path.display());

    // Show platform-specific instructions
    println!("\nNote: You may need to add this to your environment:");
    if cfg!(windows) {
        println!("  set ANDROID_HOME={}", install_path.display());
        println!("  (Or set it permanently via System Properties > Environment Variables)");
    } else {
        println!("  export ANDROID_HOME={}", install_path.display());
        println!("  (Add to ~/.bashrc, ~/.zshrc, or ~/.profile)");
    }

    // Now ensure NDK is installed
    ensure_sdkmanager(&install_path)?;

    Ok(())
}

fn show_manual_instructions(install_path: &Path) {
    let expected_path = install_path.join("cmdline-tools").join("latest");

    println!("\nPlease download Android command-line tools manually:");
    println!("  1. Go to: https://developer.android.com/studio#command-line-tools-only");
    println!("  2. Download the 'Command line tools only' package for your platform");
    println!("  3. Extract the zip to: {}", install_path.display());
    println!("  4. Ensure the structure is: {}/...", expected_path.display());
    println!("  5. Set ANDROID_HOME={}", install_path.display());
    println!("  6. Run 'orbital build android' again");
}

/// Extract a zip file using the Rust zip library
/// This avoids file locking issues with external tools like tar/unzip
fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(zip_path)
        .map_err(|e| anyhow::anyhow!("Failed to open zip file: {}", e))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| anyhow::anyhow!("Failed to read zip archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| anyhow::anyhow!("Failed to read zip entry: {}", e))?;

        let outpath = dest_dir.join(file.mangled_name());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| anyhow::anyhow!("Failed to create directory: {}", e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| anyhow::anyhow!("Failed to create directory: {}", e))?;
                }
            }
            let mut out_file = std::fs::File::create(&outpath)
                .map_err(|e| anyhow::anyhow!("Failed to create file: {}", e))?;
            std::io::copy(&mut file, &mut out_file)
                .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
        }
    }

    Ok(())
}
