mod prompt;
mod template;

use anyhow::Result;

pub fn run(
    name: Option<String>,
    package: Option<String>,
    template: Option<String>,
    android: Option<bool>,
    engine_repo: Option<String>,
    engine_branch: Option<String>,
    yes: bool,
) -> Result<()> {
    let config = if yes {
        // Non-interactive mode: use defaults or provided values
        let android_flag = android.unwrap_or(false);
        prompt::non_interactive(name, package, template, android_flag, engine_repo, engine_branch)?
    } else {
        // Interactive mode: prompt for all values
        prompt::interactive(name, package, template, android, engine_repo, engine_branch)?
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

    // Auto-run init android if requested
    if config.generate_android {
        println!("\nInitializing Android project...");

        // Save current directory and change to project directory
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(&project_dir)?;

        // Run init android
        crate::android::project::init()?;

        // Change back to original directory
        std::env::set_current_dir(original_dir)?;
    }

    println!("\nNext steps:");
    println!("  cd {}", config.project_name);

    if config.generate_android {
        println!("  orbital build android");
    } else {
        println!("  orbital build");
    }

    Ok(())
}
