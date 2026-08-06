use anyhow::Result;
use inquire::{Confirm, Select, Text};

pub struct ProjectConfig {
    pub project_name: String,
    pub package_name: String,
    pub min_sdk: u32,
    pub target_sdk: u32,
    pub template: String,
    pub generate_android: bool,
}

pub fn interactive(
    name: Option<String>,
    package: Option<String>,
    template: Option<String>,
    android: Option<bool>,
) -> Result<ProjectConfig> {
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
        None => Select::new(
            "Template:",
            vec!["minimal", "skybox", "instancing", "gltf"],
        )
        .prompt()?
        .to_string(),
    };

    Ok(ProjectConfig {
        project_name,
        package_name,
        min_sdk,
        target_sdk,
        template: template_name,
        generate_android,
    })
}

pub fn non_interactive(
    name: Option<String>,
    package: Option<String>,
    template: Option<String>,
    android: bool,
) -> Result<ProjectConfig> {
    let project_name = name.ok_or_else(|| {
        anyhow::anyhow!("Project name is required in non-interactive mode. Usage: orbital init <name>")
    })?;

    let package_name = package.unwrap_or_else(|| {
        format!("com.example.{}", project_name.replace('-', "_"))
    });

    let template_name = template.unwrap_or_else(|| "minimal".to_string());

    Ok(ProjectConfig {
        project_name,
        package_name,
        min_sdk: 21,
        target_sdk: 34,
        template: template_name,
        generate_android: android,
    })
}
