use anyhow::Result;
use inquire::{Confirm, Select, Text};

use crate::config;

pub struct ProjectConfig {
    pub project_name: String,
    pub package_name: String,
    pub min_sdk: u32,
    pub target_sdk: u32,
    pub template: String,
    pub generate_android: bool,
    pub engine_repo: String,
    pub engine_branch: String,
}

pub fn interactive(
    name: Option<String>,
    package: Option<String>,
    template: Option<String>,
    android: Option<bool>,
    engine_repo: Option<String>,
    engine_branch: Option<String>,
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

    // 5. Template selection
    let template_name = match template {
        Some(t) => t,
        None => Select::new("Template:", vec!["minimal", "skybox", "instancing", "gltf"])
            .prompt()?
            .to_string(),
    };

    // 6. Engine repository
    let engine_repo = match engine_repo {
        Some(r) => r,
        None => Text::new("Orbital engine git repo:")
            .with_default(orbital_config.engine_repo())
            .prompt()?,
    };

    // 7. Engine branch
    let engine_branch = match engine_branch {
        Some(b) => b,
        None => Text::new("Orbital engine branch:")
            .with_default(orbital_config.engine_branch())
            .prompt()?,
    };

    Ok(ProjectConfig {
        project_name,
        package_name,
        min_sdk,
        target_sdk,
        template: template_name,
        generate_android,
        engine_repo,
        engine_branch,
    })
}

pub fn non_interactive(
    name: Option<String>,
    package: Option<String>,
    template: Option<String>,
    android: bool,
    engine_repo: Option<String>,
    engine_branch: Option<String>,
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

    Ok(ProjectConfig {
        project_name,
        package_name,
        min_sdk: 21,
        target_sdk: 34,
        template: template_name,
        generate_android: android,
        engine_repo,
        engine_branch,
    })
}
