mod android;
mod config;
mod init;

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
        /// Package name (e.g., com.mycompany.mygame)
        #[arg(short, long)]
        package: Option<String>,
        /// Template to use (minimal, skybox, instancing, gltf)
        #[arg(short, long)]
        template: Option<String>,
        /// Skip interactive prompts (use defaults)
        #[arg(long)]
        yes: bool,
    },
    /// Initialize Android project for existing Orbital project
    InitAndroid,
    /// Build for a platform
    Build {
        /// Platform to build for
        #[arg(value_enum)]
        platform: Platform,
        /// Package to build (optional in standalone projects)
        #[arg(short, long)]
        package: Option<String>,
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Build, install, and run on a connected device
    Run {
        /// Platform to run on
        #[arg(value_enum)]
        platform: Platform,
        /// Package to run (optional in standalone projects)
        #[arg(short, long)]
        package: Option<String>,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum Platform {
    Android,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            name,
            package,
            template,
            yes,
        } => init::run(name, package, template, yes),
        Commands::InitAndroid => android::project::init(),
        Commands::Build {
            platform,
            package,
            release,
        } => match platform {
            Platform::Android => android::build::build(package.as_deref(), release),
        },
        Commands::Run { platform, package } => match platform {
            Platform::Android => android::run::run(package.as_deref()),
        },
    }
}
