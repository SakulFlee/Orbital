mod android;
mod assets;
mod config;
mod desktop;
mod init;
mod java;
mod tooling;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "orbital",
    bin_name = "orbital",
    about = "Build tool for the Orbital game engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Orbital project
    Init {
        /// Project name (optional, will prompt if not provided)
        name: Option<String>,
        /// Parent directory for the project (creates <path>/<name>/)
        #[arg(long)]
        parent_path: Option<PathBuf>,
        /// Exact project directory (files placed directly here)
        #[arg(long)]
        project_path: Option<PathBuf>,
        /// Package name (e.g., com.mycompany.mygame)
        #[arg(short, long)]
        package: Option<String>,
        /// Template to use (minimal, all-in-one)
        #[arg(short, long)]
        template: Option<String>,
        /// Enable Android support
        #[arg(long)]
        android: Option<bool>,
        /// Orbital engine git repo URL
        #[arg(long)]
        engine_repo: Option<String>,
        /// Orbital engine git branch
        #[arg(long)]
        engine_branch: Option<String>,
        /// Skip interactive prompts (use defaults)
        #[arg(long)]
        yes: bool,
    },
    /// Initialize Android project for existing Orbital project
    InitAndroid,
    /// Build for a platform (defaults to desktop)
    Build {
        /// Platform to build for (defaults to desktop)
        #[arg(value_enum)]
        platform: Option<Platform>,
        /// Package to build (optional in standalone projects)
        #[arg(short, long)]
        package: Option<String>,
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Build, install, and run on a connected device or emulator (defaults to desktop)
    Run {
        /// Platform to run on (defaults to desktop)
        #[arg(value_enum)]
        platform: Option<Platform>,
        /// Package to run (optional in standalone projects)
        #[arg(short, long)]
        package: Option<String>,
        /// Device serial or AVD name to run on
        #[arg(short, long)]
        device: Option<String>,
        /// Skip rebuilding and install the existing APK
        #[arg(long)]
        skip_build: bool,
        /// Skip streaming logcat after launching (attached by default)
        #[arg(long)]
        no_logcat: bool,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum Platform {
    Desktop,
    Android,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            name,
            parent_path,
            project_path,
            package,
            template,
            android,
            engine_repo,
            engine_branch,
            yes,
        } => init::run(
            name,
            parent_path,
            project_path,
            package,
            template,
            android,
            engine_repo,
            engine_branch,
            yes,
        ),
        Commands::InitAndroid => android::project::init(),
        Commands::Build {
            platform,
            package,
            release,
        } => match platform.unwrap_or(Platform::Desktop) {
            Platform::Desktop => desktop::build(release),
            Platform::Android => android::build::build(package.as_deref(), release),
        },
        Commands::Run {
            platform,
            package,
            device,
            skip_build,
            no_logcat,
        } => match platform.unwrap_or(Platform::Desktop) {
            Platform::Desktop => desktop::run(),
            Platform::Android => {
                android::run::run(package.as_deref(), device.as_deref(), skip_build, no_logcat)
            }
        },
    }
}
