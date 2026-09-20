use anyhow::{Result, bail};
use inquire::{Confirm, Select, Text};
use std::path::Path;

use crate::config;

/// Validates that the engine source (repo URL or local path) is valid.
/// `engine_repo` and `engine_path` are mutually exclusive.
fn validate_engine_source(engine_repo: &str, engine_path: Option<&str>) -> Result<()> {
    if let Some(path) = engine_path {
        // Local path mode: engine_repo should be the default (unused for Cargo.toml)
        let p = Path::new(path);
        if !p.exists() {
            bail!("engine path does not exist: {path}");
        }
        if !p.join("Crates/orbital/Cargo.toml").exists() {
            bail!("not an Orbital engine repo (missing Crates/orbital/Cargo.toml): {path}");
        }
        return Ok(());
    }

    // Remote URL mode: validate it looks like a git URL
    let valid_prefixes = ["ssh://", "https://", "http://", "git://", "git@"];
    if !valid_prefixes.iter().any(|p| engine_repo.starts_with(p)) {
        bail!(
            "engine-repo must be a remote git URL (ssh://, https://, git://, git@...), \
             or use --engine-path for a local path: {engine_repo}"
        );
    }
    Ok(())
}

pub struct ProjectConfig {
    pub project_name: String,
    pub package_name: String,
    pub min_sdk: u32,
    pub target_sdk: u32,
    pub template: String,
    pub generate_android: bool,
    pub engine_repo: String,
    pub engine_branch: String,
    pub engine_path: Option<String>,
}

pub fn interactive(
    name: Option<String>,
    package: Option<String>,
    template: Option<String>,
    android: Option<bool>,
    engine_repo: Option<String>,
    engine_branch: Option<String>,
    engine_path: Option<String>,
) -> Result<ProjectConfig> {
    let orbital_config = config::load_config()
        .map(|c| c.orbital())
        .unwrap_or_default();

    // 1. Project name
    let project_name = match name {
        Some(n) => n,
        None => Text::new("Project name:").prompt()?,
    };

    // 2. Android support (ask FIRST)
    let generate_android = match android {
        Some(a) => a,
        None => Confirm::new("Do you need Android support?")
            .with_default(true)
            .prompt()?,
    };

    // 3. Package name (only if Android, otherwise use default for Cargo.toml)
    let package_name = if generate_android {
        let default_package = format!("com.example.{}", project_name.replace('-', "_"));
        match package {
            Some(p) => p,
            None => Text::new("Package name:")
                .with_default(&default_package)
                .prompt()?,
        }
    } else {
        // Still need a package name for Cargo.toml
        match package {
            Some(p) => p,
            None => format!("com.example.{}", project_name.replace('-', "_")),
        }
    };

    // 4. SDK versions (only if Android)
    let (min_sdk, target_sdk) = if generate_android {
        let min_sdk: u32 = Text::new("Android min SDK:")
            .with_default("21")
            .prompt()?
            .parse()
            .unwrap_or(21);

        let target_sdk: u32 = Text::new("Android target SDK:")
            .with_default("34")
            .prompt()?
            .parse()
            .unwrap_or(34);

        (min_sdk, target_sdk)
    } else {
        (21, 34)
    };

    // 5. Template selection (minimal, all-in-one, or 2d)
    let template_name = match template {
        Some(t) => t,
        None => {
            let templates = vec!["minimal", "all-in-one", "2d"];
            Select::new("Select a template:", templates)
                .prompt()?
                .to_string()
        }
    };

    // 6. Engine source: local path or remote URL
    let engine_path = match engine_path {
        Some(p) => Some(p),
        None => {
            if engine_repo.is_none() {
                // No --engine-repo or --engine-path given; ask the user
                let use_local = Confirm::new("Use a local engine path instead of a git repo?")
                    .with_default(false)
                    .prompt()?;
                if use_local {
                    Some(
                        Text::new("Orbital engine local path:")
                            .with_default(orbital_config.engine_path.as_deref().unwrap_or(""))
                            .prompt()?,
                    )
                } else {
                    None
                }
            } else {
                None
            }
        }
    };

    // 7. Engine repository (only if not using local path)
    let engine_repo = if engine_path.is_some() {
        // Still need a value for the struct, use default
        orbital_config.engine_repo().to_string()
    } else {
        match engine_repo {
            Some(r) => r,
            None => Text::new("Orbital engine git repo:")
                .with_default(orbital_config.engine_repo())
                .prompt()?,
        }
    };

    // 8. Engine branch (only if not using local path)
    let engine_branch = if engine_path.is_some() {
        orbital_config.engine_branch().to_string()
    } else {
        match engine_branch {
            Some(b) => b,
            None => Text::new("Orbital engine branch:")
                .with_default(orbital_config.engine_branch())
                .prompt()?,
        }
    };

    validate_engine_source(&engine_repo, engine_path.as_deref())?;

    Ok(ProjectConfig {
        project_name,
        package_name,
        min_sdk,
        target_sdk,
        template: template_name,
        generate_android,
        engine_repo,
        engine_branch,
        engine_path,
    })
}

pub fn non_interactive(
    name: Option<String>,
    package: Option<String>,
    template: Option<String>,
    android: bool,
    engine_repo: Option<String>,
    engine_branch: Option<String>,
    engine_path: Option<String>,
) -> Result<ProjectConfig> {
    let orbital_config = config::load_config()
        .map(|c| c.orbital())
        .unwrap_or_default();

    let project_name = name.ok_or_else(|| {
        anyhow::anyhow!(
            "Project name is required in non-interactive mode. Usage: orbital init <name>"
        )
    })?;

    let package_name =
        package.unwrap_or_else(|| format!("com.example.{}", project_name.replace('-', "_")));

    let template_name = template.unwrap_or_else(|| "minimal".to_string());
    let engine_repo = engine_repo.unwrap_or_else(|| orbital_config.engine_repo().to_string());
    let engine_branch = engine_branch.unwrap_or_else(|| orbital_config.engine_branch().to_string());

    // Resolve engine_path: use provided, or fall back to config
    let engine_path = engine_path.or_else(|| orbital_config.engine_path.clone());

    validate_engine_source(&engine_repo, engine_path.as_deref())?;

    Ok(ProjectConfig {
        project_name,
        package_name,
        min_sdk: 21,
        target_sdk: 34,
        template: template_name,
        generate_android: android,
        engine_repo,
        engine_branch,
        engine_path,
    })
}
