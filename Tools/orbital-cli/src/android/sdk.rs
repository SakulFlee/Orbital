use anyhow::{Context, Result};
use inquire::{Confirm, Text};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Check if Android SDK and NDK are properly configured
pub fn ensure_android_sdk() -> Result<()> {
    // Detection chain:
    // 1. Orbital-owned SDK in the cache dir
    // 2. ANDROID_HOME
    // 3. ANDROID_SDK_ROOT
    // 4. ANDROID_SDK
    // 5. Android Studio default locations
    let sdk_path = detect_android_sdk();

    match sdk_path {
        Some(sdk_path) => {
            if sdk_path.exists() {
                println!("Android SDK found at: {}", sdk_path.display());
                set_android_home(&sdk_path);
                ensure_sdkmanager(&sdk_path)?;
                return Ok(());
            }
            println!("Configured SDK path does not exist: {}", sdk_path.display());
        }
        None => {
            println!("Android SDK not found.");
        }
    }

    // Ask user for SDK path or offer to install our own
    println!("\nAndroid SDK is required for building Android apps.");
    println!("Options:");
    println!("  1. Provide path to existing Android SDK");
    println!("  2. Install an Orbital-managed Android SDK");
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

            set_android_home(&sdk_path);

            // Check for sdkmanager
            ensure_sdkmanager(&sdk_path)?;
        }
        "2" => {
            // Install an Orbital-managed Android SDK
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

/// Detect an existing Android SDK via the standard env vars and default locations.
fn detect_android_sdk() -> Option<PathBuf> {
    // 1. Orbital-owned SDK in the cache dir
    if let Ok(dir) = crate::tooling::android_sdk_dir() {
        if dir.join("cmdline-tools").join("latest").exists() {
            return Some(dir);
        }
    }

    // 2-4. Env vars (note: ANDROID_SDK_HOME is the AVD/preferences home, NOT the SDK)
    for var in ["ANDROID_HOME", "ANDROID_SDK_ROOT", "ANDROID_SDK"] {
        if let Ok(path) = std::env::var(var) {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }
    }

    // 5. Android Studio default locations
    if cfg!(windows) {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let p = PathBuf::from(local).join("Android").join("Sdk");
            if p.exists() {
                return Some(p);
            }
        }
    } else if cfg!(target_os = "macos") {
        let home = std::env::var("HOME").ok()?;
        let p = PathBuf::from(home).join("Library").join("Android").join("sdk");
        if p.exists() {
            return Some(p);
        }
    } else {
        let home = std::env::var("HOME").ok()?;
        let p = PathBuf::from(home).join("Android").join("Sdk");
        if p.exists() {
            return Some(p);
        }
    }

    None
}

/// Sets ANDROID_HOME for this process so sdkmanager/gradle can find the SDK.
fn set_android_home(sdk_path: &Path) {
    // SAFETY: Setting an env var for the current process before spawning children.
    unsafe { std::env::set_var("ANDROID_HOME", sdk_path) };
    println!("ANDROID_HOME set to: {}", sdk_path.display());
}
/// Ensure sdkmanager is available and NDK is installed.
/// Returns the path to the NDK directory.
fn ensure_sdkmanager(sdk_path: &Path) -> Result<PathBuf> {
    let sdkmanager = sdkmanager_path(sdk_path);

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
    }

    // Ensure NDK is installed and return its path
    ensure_ndk(sdk_path)
}

/// Prints a warning that the following steps auto-accept the Android SDK licenses.
fn warn_license_acceptance() {
    println!(
        "\nNote: The following step will automatically accept the Android SDK license agreements."
    );
    println!(
        "By continuing you agree to the terms at https://developer.android.com/studio/terms"
    );
}

/// Accepts the Android SDK license agreements non-interactively by writing the
/// standard license hash files. This is the conventional approach used in CI.
fn accept_sdk_licenses(sdk_path: &Path) -> Result<()> {
    let licenses_dir = sdk_path.join("licenses");
    std::fs::create_dir_all(&licenses_dir)
        .context("Failed to create licenses directory")?;

    // Well-known accepted hashes for the Android SDK licenses
    let android_sdk_license =
        "24333f8a63b6825ea9c5514f83c2829b004d1fee\n";
    let android_sdk_preview_license =
        "84831b9409646a918e30573bab4c9c91346d8abd\n";

    std::fs::write(
        licenses_dir.join("android-sdk-license"),
        android_sdk_license,
    )
    .context("Failed to write android-sdk-license")?;

    std::fs::write(
        licenses_dir.join("android-sdk-preview-license"),
        android_sdk_preview_license,
    )
    .context("Failed to write android-sdk-preview-license")?;

    println!("Android SDK licenses accepted.");
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

    // Ask user where to install (default: Orbital-managed SDK in the cache dir)
    let default_path = crate::tooling::android_sdk_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "./android-sdk".to_string());

    let install_path = Text::new("Enter installation path:")
        .with_default(&default_path)
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
        crate::tooling::download_with_progress(&url, &temp_path, "Downloading SDK")?;

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

    // Extract the file directly into the cmdline-tools/latest/ structure.
    // The zip contains a top-level "cmdline-tools/" folder, so we strip that
    // component and write straight to cmdline-tools/latest/. This avoids a
    // directory rename, which Windows frequently blocks on freshly extracted files.
    println!("Extracting...");

    match extract_cmdline_tools(&zip_path, &install_path) {
        Ok(()) => {
            println!("Extraction complete!");
        }
        Err(e) => {
            println!("\nFailed to extract SDK tools: {}", e);
            show_manual_instructions(&install_path);
            return Ok(());
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

/// Extract the Android command-line tools zip directly into the
/// `cmdline-tools/latest/` structure.
///
/// The zip contains a top-level `cmdline-tools/` folder. We strip that leading
/// component from every entry so files land in
/// `<sdk>/cmdline-tools/latest/...` without needing a directory rename.
/// (Windows frequently locks freshly extracted files, making the rename fail.)
fn extract_cmdline_tools(zip_path: &Path, sdk_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(zip_path)
        .map_err(|e| anyhow::anyhow!("Failed to open zip file: {}", e))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| anyhow::anyhow!("Failed to read zip archive: {}", e))?;

    let latest_dir = sdk_dir.join("cmdline-tools").join("latest");
    std::fs::create_dir_all(&latest_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create {}: {}", latest_dir.display(), e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| anyhow::anyhow!("Failed to read zip entry: {}", e))?;

        // Get the entry name and strip the leading "cmdline-tools/" component
        let name = file.name().to_string();
        let stripped = strip_top_level(&name);

        // Skip the top-level directory entry itself
        if stripped.is_empty() {
            continue;
        }

        let outpath = latest_dir.join(stripped);

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| anyhow::anyhow!("Failed to create directory {}: {}", outpath.display(), e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| anyhow::anyhow!("Failed to create directory {}: {}", parent.display(), e))?;
                }
            }
            let mut out_file = std::fs::File::create(&outpath)
                .map_err(|e| anyhow::anyhow!("Failed to create file {}: {}", outpath.display(), e))?;
            std::io::copy(&mut file, &mut out_file)
                .map_err(|e| anyhow::anyhow!("Failed to write file {}: {}", outpath.display(), e))?;

            // On Unix, set executable permission for extracted files (especially
            // shell scripts like sdkmanager). The zip crate's manual extraction
            // drops Unix permission bits, leaving files at 644 (non-executable).
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(0o755))
                    .map_err(|e| anyhow::anyhow!("Failed to set permissions on {}: {}", outpath.display(), e))?;
            }
        }
    }

    Ok(())
}

/// Strips the leading directory component from an archive entry path.
/// E.g. "cmdline-tools/bin/sdkmanager.bat" -> "bin/sdkmanager.bat"
fn strip_top_level(name: &str) -> String {
    let normalized = name.replace('\\', "/");
    let mut parts = normalized.splitn(2, '/');
    let first = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("");

    // Only strip if the first component is "cmdline-tools" (case-insensitive)
    if first.eq_ignore_ascii_case("cmdline-tools") {
        rest.to_string()
    } else if rest.is_empty() {
        // Single top-level entry that isn't cmdline-tools — keep as-is
        first.to_string()
    } else {
        // Unexpected structure — keep full path
        normalized
    }
}

/// Return the expected NDK path inside the given SDK directory.
pub fn ndk_path(sdk_path: &Path, ndk_version: &str) -> PathBuf {
    sdk_path.join("ndk").join(ndk_version)
}

/// Ensure the Android NDK is installed and return its path.
///
/// Checks if the NDK directory exists at `sdk_path/ndk/<ndk_version>`.
/// If not, runs sdkmanager to install it. Returns the NDK path on success.
pub fn ensure_ndk(sdk_path: &Path) -> Result<PathBuf> {
    let ndk_version = crate::config::load_android_config()?.ndk_version().to_string();
    let ndk = ndk_path(sdk_path, &ndk_version);

    if ndk.exists() {
        println!("Android NDK found at: {}", ndk.display());
        return Ok(ndk);
    }

    println!("\nAndroid NDK not installed.");

    if Confirm::new("Install Android NDK now?")
        .with_default(true)
        .prompt()?
    {
        let sdkmanager = sdkmanager_path(sdk_path);
        if !sdkmanager.exists() {
            anyhow::bail!(
                "sdkmanager not found at {}. Cannot install NDK automatically.",
                sdkmanager.display()
            );
        }

        let java_home = crate::java::ensure_java()?;
        let ndk_package = format!("ndk;{}", ndk_version);

        warn_license_acceptance();
        accept_sdk_licenses(sdk_path)?;

        println!("Installing {}...", ndk_package);
        let status = Command::new(&sdkmanager)
            .env("JAVA_HOME", &java_home)
            .arg(&ndk_package)
            .status()
            .context("Failed to run sdkmanager")?;

        if status.success() {
            println!("NDK installed successfully!");
        } else {
            anyhow::bail!(
                "Failed to install NDK. You can try manually:\n  {} \"{}\"",
                sdkmanager.display(),
                ndk_package
            );
        }
    } else {
        anyhow::bail!(
            "Android NDK is required for building.\n\
             Install it with:\n  \
             {} \"ndk;{}\"",
            sdkmanager_path(sdk_path).display(),
            ndk_version
        );
    }

    if !ndk.exists() {
        anyhow::bail!(
            "NDK installation reported success but directory not found at: {}",
            ndk.display()
        );
    }

    Ok(ndk)
}

/// Return the currently detected SDK path, if any.
pub fn current_sdk_path() -> Option<PathBuf> {
    detect_android_sdk()
}

/// Return all candidate SDK paths, in priority order (Orbital cache, then
/// env vars, then Android Studio defaults). Some AVDs reference system
/// images that live in a different SDK than the first one detected, so the
/// emulator must search across all of them.
pub fn candidate_sdk_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(dir) = crate::tooling::android_sdk_dir() {
        push_unique(&mut paths, dir);
    }
    for var in ["ANDROID_HOME", "ANDROID_SDK_ROOT", "ANDROID_SDK"] {
        if let Ok(path) = std::env::var(var) {
            push_unique(&mut paths, PathBuf::from(path));
        }
    }

    if cfg!(windows) {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            push_unique(&mut paths, PathBuf::from(local).join("Android").join("Sdk"));
        }
    } else if cfg!(target_os = "macos") {
        let home = std::env::var("HOME").unwrap_or_default();
        push_unique(&mut paths, PathBuf::from(home).join("Library").join("Android").join("sdk"));
    } else {
        let home = std::env::var("HOME").unwrap_or_default();
        push_unique(&mut paths, PathBuf::from(home).join("Android").join("Sdk"));
    }

    paths
}

fn push_unique(paths: &mut Vec<PathBuf>, p: PathBuf) {
    if !paths.contains(&p) {
        paths.push(p);
    }
}

fn sdkmanager_path(sdk_path: &Path) -> PathBuf {
    if cfg!(windows) {
        sdk_path
            .join("cmdline-tools")
            .join("latest")
            .join("bin")
            .join("sdkmanager.bat")
    } else {
        sdk_path
            .join("cmdline-tools")
            .join("latest")
            .join("bin")
            .join("sdkmanager")
    }
}

fn avdmanager_path(sdk_path: &Path) -> PathBuf {
    if cfg!(windows) {
        sdk_path
            .join("cmdline-tools")
            .join("latest")
            .join("bin")
            .join("avdmanager.bat")
    } else {
        sdk_path
            .join("cmdline-tools")
            .join("latest")
            .join("bin")
            .join("avdmanager")
    }
}

/// The system image package to use for a new AVD: matches the project's
/// target SDK and the host's CPU architecture.
pub fn system_image_package() -> Result<String> {
    let target_sdk = crate::config::load_android_config()?.target_sdk();
    let abi = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "arm64-v8a"
    } else {
        "x86_64"
    };
    Ok(format!(
        "system-images;android-{};google_apis;{}",
        target_sdk, abi
    ))
}

/// Whether an sdkmanager package is actually installed in the SDK (by mapping
/// the package id to its on-disk directory, e.g.
/// `system-images;android-34;google_apis;x86_64` ->
/// `<sdk>/system-images/android-34/google_apis/x86_64`). A system image only
/// counts as installed once its `system.img` is present — an empty dir with
/// just an `.installer` marker means a stale/interrupted install.
pub fn package_installed(package: &str) -> bool {
    let Some(sdk) = current_sdk_path() else {
        return false;
    };
    let rel = package.replace(';', "/");
    let dir = sdk.join(&rel);
    if !dir.exists() {
        return false;
    }
    if package.starts_with("system-images;") {
        dir.join("system.img").exists()
    } else {
        true
    }
}

/// Install the given sdkmanager packages (e.g. "emulator", system images).
/// Accepts the SDK licenses first (a warning is printed).
pub fn sdkmanager_install(packages: &[String]) -> Result<()> {
    let sdk_path = current_sdk_path().context("Android SDK not found")?;
    let java_home = crate::java::ensure_java()?;
    let sdkmanager = sdkmanager_path(&sdk_path);

    if !sdkmanager.exists() {
        anyhow::bail!("sdkmanager not found at {}", sdkmanager.display());
    }

    warn_license_acceptance();
    accept_sdk_licenses(&sdk_path)?;

    for pkg in packages {
        println!("Installing {}...", pkg);
        let status = Command::new(&sdkmanager)
            .env("JAVA_HOME", &java_home)
            .arg(pkg)
            .status()
            .with_context(|| format!("Failed to run sdkmanager for {}", pkg))?;
        if !status.success() {
            anyhow::bail!("sdkmanager failed to install {}", pkg);
        }
    }

    Ok(())
}

/// Create an AVD using avdmanager. Answers "no" to the custom hardware
/// profile prompt so creation is fully non-interactive.
pub fn create_avd(name: &str, system_image: &str) -> Result<()> {
    let sdk_path = current_sdk_path().context("Android SDK not found")?;
    let java_home = crate::java::ensure_java()?;
    let avdmanager = avdmanager_path(&sdk_path);

    if !avdmanager.exists() {
        anyhow::bail!("avdmanager not found at {}", avdmanager.display());
    }

    println!("Creating AVD '{}'...", name);
    let mut child = Command::new(&avdmanager)
        .env("JAVA_HOME", &java_home)
        .args([
            "create",
            "avd",
            "--name",
            name,
            "--package",
            system_image,
            "--force",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to run avdmanager")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"no\n")?;
    }

    let status = child.wait().context("Failed to wait for avdmanager")?;
    if !status.success() {
        anyhow::bail!("avdmanager create avd failed");
    }

    println!("AVD '{}' created.", name);
    Ok(())
}

