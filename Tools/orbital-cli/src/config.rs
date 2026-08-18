use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct OrbitalConfig {
    pub orbital: Option<OrbitalGeneral>,
    pub android: Option<AndroidConfig>,
}

impl OrbitalConfig {
    pub fn orbital(&self) -> OrbitalGeneral {
        self.orbital.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OrbitalGeneral {
    pub engine_repo: Option<String>,
    pub engine_branch: Option<String>,
}

impl OrbitalGeneral {
    pub fn engine_repo(&self) -> &str {
        self.engine_repo
            .as_deref()
            .unwrap_or("https://forgejo.sakul-flee.de/SakulFlee/Orbital.git")
    }

    pub fn engine_branch(&self) -> &str {
        self.engine_branch.as_deref().unwrap_or("android")
    }
}

#[derive(Debug, Deserialize)]
pub struct AndroidConfig {
    pub package: Option<String>,
    pub min_sdk: Option<u32>,
    pub target_sdk: Option<u32>,
    pub targets: Option<Vec<String>>,
    pub apk_mode: Option<String>,
    pub ndk_version: Option<String>,
    pub screen_orientation: Option<String>,
}

impl Default for AndroidConfig {
    fn default() -> Self {
        Self {
            package: Some("de.sakulflee.orbital".to_string()),
            min_sdk: Some(21),
            target_sdk: Some(34),
            targets: None,
            apk_mode: None,
            ndk_version: None,
            screen_orientation: None,
        }
    }
}

impl AndroidConfig {
    pub fn package_name(&self) -> &str {
        self.package.as_deref().unwrap_or("de.sakulflee.orbital")
    }

    pub fn min_sdk(&self) -> u32 {
        self.min_sdk.unwrap_or(21)
    }

    pub fn target_sdk(&self) -> u32 {
        self.target_sdk.unwrap_or(34)
    }

    pub fn targets(&self) -> Vec<String> {
        self.targets.clone().unwrap_or_else(|| {
            vec![
                "arm64-v8a".to_string(),
                "armeabi-v7a".to_string(),
                "x86_64".to_string(),
                "x86".to_string(),
            ]
        })
    }

    pub fn apk_mode(&self) -> &str {
        self.apk_mode.as_deref().unwrap_or("multiarch")
    }

    pub fn ndk_version(&self) -> &str {
        self.ndk_version.as_deref().unwrap_or("26.2.11394342")
    }

    pub fn screen_orientation(&self) -> &str {
        self.screen_orientation.as_deref().unwrap_or("landscape")
    }
}

/// Find the project root by looking for Orbital.toml
pub fn find_project_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir().context("Failed to get current directory")?;

    loop {
        if dir.join("Orbital.toml").exists() {
            return Ok(dir);
        }

        dir = dir
            .parent()
            .context("Not an Orbital project (no Orbital.toml found)")?
            .to_path_buf();
    }
}

/// Load the Orbital.toml config
pub fn load_config() -> Result<OrbitalConfig> {
    let project_root = find_project_root()?;
    let config_path = project_root.join("Orbital.toml");

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;

    let config: OrbitalConfig = toml::from_str(&content).context("Failed to parse Orbital.toml")?;

    Ok(config)
}

/// Load Android config with defaults
pub fn load_android_config() -> Result<AndroidConfig> {
    let config = load_config()?;
    let android = config.android.unwrap_or_default();
    if let Some(orientation) = &android.screen_orientation {
        if !VALID_SCREEN_ORIENTATIONS.contains(&orientation.as_str()) {
            anyhow::bail!(
                "Invalid screen_orientation \"{orientation}\" in Orbital.toml. \
                 Valid values: {}",
                VALID_SCREEN_ORIENTATIONS.join(", ")
            );
        }
    }
    Ok(android)
}

/// Valid values for the `android:screenOrientation` manifest attribute.
const VALID_SCREEN_ORIENTATIONS: &[&str] = &[
    "unspecified",
    "behind",
    "landscape",
    "portrait",
    "reverseLandscape",
    "reversePortrait",
    "sensorLandscape",
    "sensorPortrait",
    "userLandscape",
    "userPortrait",
    "sensor",
    "fullSensor",
    "nosensor",
    "user",
    "fullUser",
    "locked",
];

/// Get the package name from the current directory's Cargo.toml
pub fn get_package_name() -> Result<String> {
    let cargo_toml_path = Path::new("Cargo.toml");
    if !cargo_toml_path.exists() {
        anyhow::bail!("No Cargo.toml found in current directory");
    }

    let content = std::fs::read_to_string(cargo_toml_path).context("Failed to read Cargo.toml")?;

    // Parse [package] name
    let mut in_package_section = false;
    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[package]" {
            in_package_section = true;
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_package_section = false;
            continue;
        }

        if in_package_section
            && trimmed.starts_with("name")
            && let Some(value) = trimmed.split_once('=')
        {
            let value = value.1.trim().trim_matches('"');
            return Ok(value.to_string());
        }
    }

    anyhow::bail!("No [package] name found in Cargo.toml")
}

/// Get the lib name from the current directory's Cargo.toml
pub fn get_lib_name() -> Result<String> {
    let cargo_toml_path = Path::new("Cargo.toml");
    if !cargo_toml_path.exists() {
        anyhow::bail!("No Cargo.toml found in current directory");
    }

    let content = std::fs::read_to_string(cargo_toml_path).context("Failed to read Cargo.toml")?;

    // Parse [lib] name
    let mut in_lib_section = false;
    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[lib]" {
            in_lib_section = true;
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_lib_section = false;
            continue;
        }

        if in_lib_section
            && trimmed.starts_with("name")
            && let Some(value) = trimmed.split_once('=')
        {
            let value = value.1.trim().trim_matches('"');
            return Ok(value.to_string());
        }
    }

    // Fallback: use package name
    get_package_name()
}

/// Find a package's path in the workspace
pub fn find_package_path(package_name: &str) -> Result<PathBuf> {
    let project_root = find_project_root()?;
    let cargo_toml_path = project_root.join("Cargo.toml");

    let content = std::fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;

    // Check if this is a workspace
    if !content.contains("[workspace]") {
        anyhow::bail!("Not a workspace. Use --package only in workspace projects.");
    }

    // Parse workspace members
    let mut in_workspace_section = false;
    let mut in_members_list = false;
    let mut members = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[workspace]" {
            in_workspace_section = true;
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_workspace_section = false;
            in_members_list = false;
            continue;
        }

        if in_workspace_section && trimmed.starts_with("members") {
            in_members_list = true;
            // Check for inline list
            if let Some(list_start) = trimmed.find('[')
                && let Some(list_end) = trimmed.find(']')
            {
                let list = &trimmed[list_start + 1..list_end];
                for item in list.split(',') {
                    let item = item.trim().trim_matches('"');
                    members.push(item.to_string());
                }
                in_members_list = false;
            }
            continue;
        }

        if in_members_list {
            if trimmed.starts_with('"') || trimmed.starts_with('\'') {
                let member = trimmed.trim_matches(|c| c == '"' || c == '\'');
                members.push(member.to_string());
            } else if trimmed == "]" {
                in_members_list = false;
            }
        }
    }

    // Find the package in members
    for member in &members {
        let member_path = project_root.join(member);
        if !member_path.exists() {
            continue;
        }

        let member_cargo = member_path.join("Cargo.toml");
        if !member_cargo.exists() {
            continue;
        }

        let member_content = std::fs::read_to_string(&member_cargo)
            .with_context(|| format!("Failed to read {}", member_cargo.display()))?;

        // Check if this member has the package name
        let mut in_package_section = false;
        for line in member_content.lines() {
            let trimmed = line.trim();

            if trimmed == "[package]" {
                in_package_section = true;
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                in_package_section = false;
                continue;
            }

            if in_package_section
                && trimmed.starts_with("name")
                && let Some(value) = trimmed.split_once('=')
            {
                let value = value.1.trim().trim_matches('"');
                if value == package_name {
                    return Ok(member_path);
                }
            }
        }
    }

    anyhow::bail!("Package '{}' not found in workspace", package_name)
}

/// Find a package's lib name in the workspace
pub fn find_package_lib_name(package_name: &str) -> Result<String> {
    let package_path = find_package_path(package_name)?;
    let cargo_toml_path = package_path.join("Cargo.toml");

    let content = std::fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;

    // Parse [lib] name
    let mut in_lib_section = false;
    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[lib]" {
            in_lib_section = true;
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_lib_section = false;
            continue;
        }

        if in_lib_section
            && trimmed.starts_with("name")
            && let Some(value) = trimmed.split_once('=')
        {
            let value = value.1.trim().trim_matches('"');
            return Ok(value.to_string());
        }
    }

    // Fallback: use package name
    Ok(package_name.to_string())
}
