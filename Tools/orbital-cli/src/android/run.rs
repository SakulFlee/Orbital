use anyhow::{Context, Result};
use inquire::{Confirm, Select};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::android::device::{self, Device};
use crate::config;

pub fn run(
    package_name: Option<&str>,
    device_serial: Option<&str>,
    skip_build: bool,
    logcat: bool,
) -> Result<()> {
    // First build (unless skipped)
    if !skip_build {
        super::build::build(package_name, false)?;
    }

    let project_root = config::find_project_root()?;
    let android_dir = project_root.join("Android");

    let android_config = config::load_android_config()?;
    let package = if let Some(pkg) = package_name {
        pkg.to_string()
    } else {
        config::get_package_name()?
    };
    let full_package = format!("{}.{}", android_config.package_name(), package);

    // Resolve adb and pick a target (device, emulator, or create one)
    let adb = device::resolve_adb()?;
    let serial = select_device(&adb, device_serial)?;

    // Pick the APK that matches the device ABI when there are multiple
    let apk_path = find_apk(&android_dir, device::device_abi(&adb, &serial).ok().as_deref())?;

    println!("\nInstalling on {}...", serial);
    let status = Command::new(&adb)
        .args(["-s", &serial, "install", "-r"])
        .arg(&apk_path)
        .status()
        .context("Failed to run adb install")?;
    if !status.success() {
        anyhow::bail!("adb install failed");
    }

    println!("Launching app...");
    let status = Command::new(&adb)
        .args([
            "-s", &serial, "shell", "am", "start", "-n",
        ])
        .arg(format!("{}/android.app.NativeActivity", full_package))
        .status()
        .context("Failed to run adb shell am start")?;
    if !status.success() {
        anyhow::bail!("Failed to launch app");
    }

    println!("\nApp launched successfully on {}!", serial);

    if logcat {
        println!("Streaming logcat (Ctrl+C to stop)...\n");
        Command::new(&adb)
            .args(["-s", &serial, "logcat", "-s", "rust_std_out"])
            .status()
            .context("Failed to run adb logcat")?;
    } else {
        println!("To view logs: adb -s {} logcat -s rust_std_out", serial);
    }

    Ok(())
}

/// Choose the device or emulator to run on.
///
/// Priority: explicit `--device` flag, then connected devices (prompt if
/// multiple), then existing AVDs (prompt if multiple), then create a new AVD.
fn select_device(adb: &Path, requested: Option<&str>) -> Result<String> {
    let devices = device::list_devices(adb)?;

    if let Some(requested) = requested {
        return select_requested(adb, &devices, requested);
    }

    if !devices.is_empty() {
        let selected = if devices.len() == 1 {
            println!("Using connected device: {}", devices[0].serial);
            devices[0].clone()
        } else {
            prompt_device(&devices)?
        };
        if selected.is_emulator {
            device::wait_for_boot(adb, &selected.serial)?;
        }
        return Ok(selected.serial);
    }

    // No connected devices: look for existing AVDs
    println!("\nNo connected devices found.");

    // If no emulator package is installed, go straight to setup.
    if device::resolve_emulator()?.is_none() {
        println!("No emulator is installed.");
        let avd = setup_new_avd()?;
        let emulator = device::resolve_emulator()?
            .context("Emulator was installed but the binary could not be found")?;
        let serial = device::boot_avd(&emulator, &avd, adb)?;
        device::wait_for_boot(adb, &serial)?;
        return Ok(serial);
    }

    let emulator = device::resolve_emulator()?.unwrap();
    let avds = device::list_avds(&emulator)?;

    let avd = if avds.is_empty() {
        setup_new_avd()?
    } else if avds.len() == 1 {
        println!("Using existing AVD: {}", avds[0]);
        avds[0].clone()
    } else {
        Select::new("Select an emulator to boot:", avds).prompt()?
    };

    // setup_new_avd may have just installed the emulator package; re-resolve.
    let emulator = device::resolve_emulator()?
        .context("Emulator was installed but the binary could not be found")?;

    match device::check_acceleration(&emulator) {
        Ok(false) => println!(
            "Warning: no hardware acceleration detected; the emulator may run slowly."
        ),
        Ok(true) => {}
        Err(_) => {}
    }

    let serial = device::boot_avd(&emulator, &avd, adb)?;
    device::wait_for_boot(adb, &serial)?;
    Ok(serial)
}

/// Handle an explicit `--device` request: a device serial or an AVD name.
fn select_requested(adb: &Path, devices: &[Device], requested: &str) -> Result<String> {
    if let Some(d) = devices.iter().find(|d| d.serial == requested) {
        if d.is_emulator {
            device::wait_for_boot(adb, requested)?;
        }
        return Ok(requested.to_string());
    }

    // Not a connected device — maybe it's an AVD to boot.
    let emulator = match device::resolve_emulator()? {
        Some(e) => e,
        None => anyhow::bail!(
            "No emulator is installed. Run 'orbital run android' without --device to set one up."
        ),
    };
    let avds = device::list_avds(&emulator)?;
    if avds.iter().any(|a| a == requested) {
        let serial = device::boot_avd(&emulator, requested, adb)?;
        device::wait_for_boot(adb, &serial)?;
        return Ok(serial);
    }

    anyhow::bail!(
        "Device or AVD '{}' not found. Use 'adb devices' to list devices and '{} -list-avds' to list emulators.",
        requested,
        emulator.display()
    )
}

fn prompt_device(devices: &[Device]) -> Result<Device> {
    let options: Vec<String> = devices
        .iter()
        .map(|d| {
            if d.is_emulator {
                format!("{} (emulator)", d.serial)
            } else {
                format!("{} (device)", d.serial)
            }
        })
        .collect();

    let idx = Select::new("Select a device:", options)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Failed to select device: {}", e))?;

    Ok(devices
        .iter()
        .find(|d| {
            let label = if d.is_emulator {
                format!("{} (emulator)", d.serial)
            } else {
                format!("{} (device)", d.serial)
            };
            label == idx
        })
        .cloned()
        .unwrap())
}

/// Install the emulator + system image and create a fresh AVD, after a single
/// confirmation (this is a ~2 GB download).
fn setup_new_avd() -> Result<String> {
    let system_image = crate::android::sdk::system_image_package()?;

    println!("\nNo emulator (AVD) is configured.");
    println!("To run on an emulator we need to install:");
    println!("  - Android Emulator (~400 MB)");
    println!("  - System image {} (~1.5 GB)", system_image);
    println!("  - Create a new AVD");

    let confirmed = Confirm::new("Install these and create an emulator now?")
        .with_default(true)
        .prompt()?;
    if !confirmed {
        anyhow::bail!(
            "Emulator setup declined. Connect a device with USB debugging enabled or run again to set up an emulator."
        );
    }

    crate::android::sdk::sdkmanager_install(&["emulator".to_string(), system_image.clone()])?;
    crate::android::sdk::create_avd("orbital-default", &system_image)?;

    Ok("orbital-default".to_string())
}

/// Locate the APK to install. If several exist (e.g. per-ABI single APKs),
/// prefer the one matching the device ABI, then the multiarch one.
fn find_apk(android_dir: &Path, device_abi: Option<&str>) -> Result<PathBuf> {
    for sub in ["debug", "release"] {
        let dir = android_dir
            .join("app")
            .join("build")
            .join("outputs")
            .join("apk")
            .join(sub);

        if !dir.exists() {
            continue;
        }

        let mut apks: Vec<PathBuf> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "apk").unwrap_or(false))
            .collect();

        if apks.is_empty() {
            continue;
        }
        if apks.len() == 1 {
            return Ok(apks.remove(0));
        }

        if let Some(abi) = device_abi
            && let Some(p) = apks.iter().find(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().contains(abi))
                    .unwrap_or(false)
            })
        {
            return Ok(p.clone());
        }

        // Prefer the multiarch APK
        if let Some(p) = apks.iter().find(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().starts_with("app-debug"))
                .unwrap_or(false)
        }) {
            return Ok(p.clone());
        }

        // Fall back to the largest APK
        apks.sort_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0));
        return Ok(apks.last().unwrap().clone());
    }

    anyhow::bail!(
        "No APK found in {}. Run 'orbital build android' first.",
        android_dir
            .join("app")
            .join("build")
            .join("outputs")
            .join("apk")
            .display()
    )
}
