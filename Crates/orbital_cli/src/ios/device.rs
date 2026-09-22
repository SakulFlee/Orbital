use anyhow::{Context, Result};
use std::process::Command;

/// A detected iOS simulator.
#[derive(Debug, Clone)]
pub struct Simulator {
    pub udid: String,
    pub name: String,
    pub device_type: String,
    pub state: String,
}

/// List available iOS simulators via `xcrun simctl list devices`.
pub fn list_simulators() -> Result<Vec<Simulator>> {
    let output = Command::new("xcrun")
        .args(["simctl", "list", "devices", "available", "--json"])
        .output()
        .context("Failed to run xcrun simctl. Is Xcode installed?")?;

    if !output.status.success() {
        anyhow::bail!(
            "xcrun simctl failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse simctl JSON output")?;

    let mut simulators = Vec::new();

    if let Some(devices) = json.get("devices") {
        for (_runtime, device_list) in devices.as_object().unwrap_or(&serde_json::Map::new()) {
            if let Some(arr) = device_list.as_array() {
                for device in arr {
                    let udid = device["udid"].as_str().unwrap_or("").to_string();
                    let name = device["name"].as_str().unwrap_or("").to_string();
                    let state = device["state"].as_str().unwrap_or("").to_string();

                    // Only include available (bootable) simulators
                    if state == "Booted" || state == "Shutdown" {
                        simulators.push(Simulator {
                            udid,
                            name,
                            device_type: String::new(),
                            state,
                        });
                    }
                }
            }
        }
    }

    Ok(simulators)
}

/// Boot a simulator by UDID.
pub fn boot_simulator(udid: &str) -> Result<()> {
    // Check if already booted
    let sim = find_simulator(udid)?;
    if sim.state == "Booted" {
        println!("Simulator '{}' is already booted.", sim.name);
        return Ok(());
    }

    println!("Booting simulator '{}'...", sim.name);
    let status = Command::new("xcrun")
        .args(["simctl", "boot", udid])
        .status()
        .context("Failed to boot simulator")?;

    if !status.success() {
        anyhow::bail!("Failed to boot simulator {}", udid);
    }

    // Wait for boot
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("Simulator '{}' booted.", sim.name);

    Ok(())
}

/// Shutdown a simulator by UDID.
pub fn shutdown_simulator(udid: &str) -> Result<()> {
    let status = Command::new("xcrun")
        .args(["simctl", "shutdown", udid])
        .status()
        .context("Failed to shutdown simulator")?;

    if !status.success() {
        // Might already be shutdown, that's fine
        println!("Note: Simulator may already be shut down.");
    }

    Ok(())
}

/// Find a specific simulator by UDID.
pub fn find_simulator(udid: &str) -> Result<Simulator> {
    let simulators = list_simulators()?;
    simulators
        .into_iter()
        .find(|s| s.udid == udid)
        .with_context(|| format!("Simulator with UDID '{}' not found", udid))
}

/// Interactive simulator selection prompt.
pub fn select_simulator(requested: Option<&str>) -> Result<Simulator> {
    let simulators = list_simulators()?;

    if simulators.is_empty() {
        anyhow::bail!(
            "No iOS simulators found. Install simulators via Xcode → Settings → Platforms."
        );
    }

    if let Some(udid) = requested {
        return find_simulator(udid);
    }

    if simulators.len() == 1 {
        println!("Using simulator: {}", simulators[0].name);
        return Ok(simulators[0].clone());
    }

    // Prompt user to select
    println!("\nAvailable simulators:");
    for (i, sim) in simulators.iter().enumerate() {
        let state_marker = if sim.state == "Booted" {
            " (running)"
        } else {
            ""
        };
        println!(
            "  [{}] {} ({}){}",
            i + 1,
            sim.name,
            &sim.udid[..8],
            state_marker
        );
    }

    print!("Select simulator [1]: ");
    use std::io::Write;
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let index: usize = input.trim().parse::<usize>().unwrap_or(1).saturating_sub(1);
    let index = index.min(simulators.len() - 1);

    Ok(simulators[index].clone())
}
