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
    let repo_url = "https://dl.google.com/android/repository/repository2-3.xml";

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

    // Fetch the repository index
    println!("Fetching latest version from Google's repository...");

    let output = if cfg!(windows) {
        Command::new("powershell")
            .args(["-Command", &format!("(Invoke-WebRequest -Uri '{}' -UseBasicParsing).Content", repo_url)])
            .output()
    } else {
        Command::new("curl")
            .args(["-s", repo_url])
            .output()
    };

    let output = output.context("Failed to fetch repository index. Is curl/powershell installed?")?;

    if !output.status.success() {
        anyhow::bail!("Failed to fetch repository index");
    }

    let xml = String::from_utf8_lossy(&output.stdout);

    // Parse the XML to find the latest command-line tools version
    // Look for patterns like: commandlinetools-{platform}-{version}_latest.zip
    let mut latest_version: Option<u64> = None;
    let mut latest_url: Option<String> = None;

    for line in xml.lines() {
        let line = line.trim();

        // Look for URL lines containing commandlinetools
        if line.starts_with("<url>") && line.contains("commandlinetools") && line.contains(platform_suffix) {
            // Extract the URL content
            if let Some(url_start) = line.find("<url>") {
                if let Some(url_end) = line.find("</url>") {
                    let url = &line[url_start + 5..url_end];

                    // Extract version number from URL like "commandlinetools-linux-15859902_latest.zip"
                    if let Some(pos) = url.find(&format!("{}-", platform_suffix)) {
                        let after_platform = &url[pos + platform_suffix.len() + 1..];
                        if let Some(dash_pos) = after_platform.find('-') {
                            let version_str = &after_platform[..dash_pos];
                            if let Ok(version) = version_str.parse::<u64>() {
                                if latest_version.is_none() || version > latest_version.unwrap() {
                                    latest_version = Some(version);
                                    latest_url = Some(format!("https://dl.google.com/android/repository/{}", url));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    match (latest_version, latest_url) {
        (Some(version), Some(url)) => {
            println!("Found latest version: {}", version);
            Ok((url, "commandlinetools.zip".to_string()))
        }
        _ => {
            // Fallback to a known working version if parsing fails
            println!("Could not determine latest version, using fallback...");
            let fallback_url = format!(
                "https://dl.google.com/android/repository/commandlinetools-{}-15859902_latest.zip",
                platform_suffix
            );
            Ok((fallback_url, "commandlinetools.zip".to_string()))
        }
    }
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

    // Download the file
    println!("Downloading from: {}", url);

    let download_result = if cfg!(windows) {
        Command::new("powershell")
            .args(["-Command", &format!("Invoke-WebRequest -Uri '{}' -OutFile '{}\\{}' -UseBasicParsing", url, install_path.display(), filename)])
            .output()
    } else {
        Command::new("curl")
            .args(["-L", "-o", &install_path.join(&filename).to_string_lossy(), &url])
            .output()
    };

    match download_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("\nDownload failed: {}", stderr);
                show_manual_instructions(&install_path);
                return Ok(());
            }
        }
        Err(e) => {
            println!("\nFailed to run download command: {}", e);
            show_manual_instructions(&install_path);
            return Ok(());
        }
    }

    // Extract the file
    println!("Extracting...");

    let zip_path = install_path.join(filename);
    if !zip_path.exists() {
        println!("\nDownloaded file not found.");
        show_manual_instructions(&install_path);
        return Ok(());
    }

    let status = if cfg!(windows) {
        Command::new("powershell")
            .args(["-Command", &format!("Expand-Archive -Path '{}' -DestinationPath '{}' -Force", zip_path.display(), install_path.display())])
            .status()
    } else {
        Command::new("unzip")
            .args(["-o", &zip_path.to_string_lossy(), "-d", &install_path.to_string_lossy()])
            .status()
    };

    match status {
        Ok(s) if s.success() => {
            // Move cmdline-tools to the right place
            let cmdline_tools = install_path.join("cmdline-tools");
            let latest = install_path.join("cmdline-tools").join("latest");

            if cmdline_tools.exists() && !latest.exists() {
                std::fs::rename(&cmdline_tools, &latest)?;
            }

            // Clean up zip file
            std::fs::remove_file(&zip_path).ok();

            // Set ANDROID_HOME
            // SAFETY: We're setting an environment variable for the current process
            unsafe { std::env::set_var("ANDROID_HOME", &install_path) };
            println!("\nSDK installed to: {}", install_path.display());
            println!("ANDROID_HOME set to: {}", install_path.display());

            println!("\nNote: You may need to add this to your shell profile:");
            println!("  export ANDROID_HOME={}", install_path.display());

            // Now ensure NDK is installed
            ensure_sdkmanager(&install_path)?;
        }
        _ => {
            println!("\nFailed to extract SDK tools.");
            show_manual_instructions(&install_path);
        }
    }

    Ok(())
}

fn show_manual_instructions(install_path: &Path) {
    println!("\nPlease download Android command-line tools manually:");
    println!("  1. Go to: https://developer.android.com/studio#command-line-tools-only");
    println!("  2. Download the 'Command line tools only' package for your platform");
    println!("  3. Extract the zip to: {}", install_path.display());
    println!("  4. Ensure the structure is: {}/cmdline-tools/latest/...", install_path.display());
    println!("  5. Set ANDROID_HOME={}", install_path.display());
    println!("  6. Run 'orbital build android' again");
}
