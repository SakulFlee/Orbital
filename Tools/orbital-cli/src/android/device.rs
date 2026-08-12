use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A detected Android device (physical or emulator).
#[derive(Debug, Clone)]
pub struct Device {
    pub serial: String,
    pub is_emulator: bool,
}

/// Resolve the adb binary: prefer the SDK's platform-tools, fall back to PATH.
pub fn resolve_adb() -> Result<PathBuf> {
    if let Some(sdk) = crate::android::sdk::current_sdk_path() {
        let adb = sdk.join("platform-tools").join(exe_name("adb"));
        if adb.exists() {
            return Ok(adb);
        }
    }
    Ok(PathBuf::from("adb"))
}

/// Resolve the emulator binary, or `None` if no emulator package is installed
/// and none is on PATH.
pub fn resolve_emulator() -> Result<Option<PathBuf>> {
    if let Some(sdk) = crate::android::sdk::current_sdk_path() {
        let emulator = sdk.join("emulator").join(exe_name("emulator"));
        if emulator.exists() {
            return Ok(Some(emulator));
        }
    }
    if command_on_path("emulator") {
        return Ok(Some(PathBuf::from("emulator")));
    }
    Ok(None)
}

/// Whether `name` resolves to an executable on PATH.
fn command_on_path(name: &str) -> bool {
    let exe = exe_name(name);
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            if dir.join(&exe).exists() {
                return true;
            }
        }
    }
    false
}

fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    }
}

/// List online devices via `adb devices`.
pub fn list_devices(adb: &Path) -> Result<Vec<Device>> {
    let output = Command::new(adb)
        .arg("devices")
        .output()
        .with_context(|| format!("Failed to run {} devices", adb.display()))?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    // Skip the "List of devices attached" header line.
    for line in text.lines().skip(1) {
        let mut parts = line.split_whitespace();
        let serial = match parts.next() {
            Some(s) => s,
            None => continue,
        };
        let state = parts.next().unwrap_or("");
        if state != "device" {
            continue;
        }
        devices.push(Device {
            serial: serial.to_string(),
            is_emulator: serial.starts_with("emulator-"),
        });
    }

    Ok(devices)
}

/// List available AVDs via `emulator -list-avds`.
pub fn list_avds(emulator: &Path) -> Result<Vec<String>> {
    let output = Command::new(emulator)
        .arg("-list-avds")
        .output()
        .with_context(|| format!("Failed to run {} -list-avds", emulator.display()))?;

    let mut avds = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("INFO") {
            continue;
        }
        avds.push(line.to_string());
    }

    Ok(avds)
}

/// Check whether hardware acceleration is available for the emulator.
pub fn check_acceleration(emulator: &Path) -> Result<bool> {
    match Command::new(emulator).arg("-accel-check").status() {
        Ok(status) => Ok(status.success()),
        Err(_) => Ok(false),
    }
}

/// Query the device's CPU ABI (e.g. arm64-v8a, x86_64).
pub fn device_abi(adb: &Path, serial: &str) -> Result<String> {
    let output = Command::new(adb)
        .args(["-s", serial, "shell", "getprop", "ro.product.cpu.abi"])
        .output()
        .context("Failed to query device ABI")?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Whether the device has finished booting (sys.boot_completed == 1).
pub fn is_booted(adb: &Path, serial: &str) -> bool {
    let Ok(output) = Command::new(adb)
        .args(["-s", serial, "shell", "getprop", "sys.boot_completed"])
        .output()
    else {
        return false;
    };
    String::from_utf8_lossy(&output.stdout).trim() == "1"
}

/// Wait until the device is online and has finished booting.
pub fn wait_for_boot(adb: &Path, serial: &str) -> Result<()> {
    println!("Waiting for device {}...", serial);

    // Blocks until the device is online (no-op if already online).
    let _ = Command::new(adb)
        .args(["-s", serial, "wait-for-device"])
        .status();

    let timeout = Duration::from_secs(180);
    let start = Instant::now();
    while !is_booted(adb, serial) {
        if start.elapsed() > timeout {
            anyhow::bail!(
                "Timed out waiting for device {} to finish booting",
                serial
            );
        }
        std::thread::sleep(Duration::from_secs(2));
    }

    println!("Device {} booted.", serial);
    Ok(())
}

/// Start the given AVD in the background and return its serial once it shows
/// up in `adb devices`.
pub fn boot_avd(emulator: &Path, name: &str, adb: &Path) -> Result<String> {
    // Track existing emulator serials so we can detect the newly booted one.
    let before: Vec<String> = list_devices(adb)?
        .into_iter()
        .filter(|d| d.is_emulator)
        .map(|d| d.serial)
        .collect();

    println!("Starting emulator '{}'...", name);
    Command::new(emulator)
        .args(["-avd", name, "-no-snapshot", "-no-audio"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to start emulator: {}", emulator.display()))?;

    // Wait for a new emulator-* device to appear.
    let timeout = Duration::from_secs(120);
    let start = Instant::now();
    loop {
        if start.elapsed() > timeout {
            anyhow::bail!("Timed out waiting for emulator '{}' to start", name);
        }
        for d in list_devices(adb)? {
            if d.is_emulator && !before.contains(&d.serial) {
                return Ok(d.serial);
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}
