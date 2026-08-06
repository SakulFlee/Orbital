mod prompt;
mod template;

use anyhow::Result;

pub fn run(
    name: Option<String>,
    package: Option<String>,
    template: Option<String>,
    yes: bool,
) -> Result<()> {
    let config = if yes {
        // Non-interactive mode: use defaults or provided values
        prompt::non_interactive(name, package, template)?
    } else {
        // Interactive mode: prompt for all values
        prompt::interactive(name, package, template)?
    };

    println!("\nGenerating project...");

    // Create project directory
    let project_dir = std::env::current_dir()?.join(&config.project_name);
    if project_dir.exists() {
        anyhow::bail!(
            "Directory '{}' already exists. Please choose a different name.",
            config.project_name
        );
    }

    std::fs::create_dir_all(&project_dir)?;

    // Generate project files
    template::generate_project(&project_dir, &config)?;

    println!(
        "\nProject '{}' created successfully!",
        config.project_name
    );
    println!("\nNext steps:");
    println!("  cd {}", config.project_name);

    if config.generate_android {
        println!("  orbital init android");
        println!("  orbital build android");
    } else {
        println!("  # Add orbital dependency to Cargo.toml");
        println!("  # Run 'orbital init android' later to add Android support");
    }

    Ok(())
}
