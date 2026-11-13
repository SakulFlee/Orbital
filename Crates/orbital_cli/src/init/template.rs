use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use std::process::Command;

use super::prompt::ProjectConfig;

pub fn generate_project(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    fs::create_dir_all(project_dir.join("src"))?;

    generate_lib_rs(project_dir, config)?;
    generate_cargo_toml(project_dir, config)?;
    generate_orbital_toml(project_dir, config)?;
    generate_devenv(project_dir, config)?;
    generate_main_rs(project_dir, config)?;
    generate_gitignore(project_dir)?;

    // Create the Assets/ directory for project-owned assets. Desktop builds
    // resolve runtime paths against <cwd>/Assets and Android bundles this
    // directory into the APK.
    fs::create_dir_all(project_dir.join("Assets"))?;
    fs::write(project_dir.join("Assets").join(".gitkeep"), "")
        .context("Failed to write Assets/.gitkeep")?;

    // The all-in-one template ships its own scene and helmet model. Embedding
    // them keeps generated projects runnable when the CLI was installed from a
    // git checkout, where no Examples/ directory exists on disk.
    if config.template == "all-in-one" {
        write_all_in_one_assets(project_dir)?;
    }

    Ok(())
}

/// Writes a project `.gitignore` so Nix's Git source filtering does not copy
/// build outputs such as `target/` or `Android/` into the store.
fn generate_gitignore(project_dir: &Path) -> Result<()> {
    const GITIGNORE: &str = r#"# Rust
/target/

# Generated Android project
/Android/

# Direnv and Cargo state
/.direnv/
/.cargo/

# Logs
*.log
Logs/

"#;

    fs::write(project_dir.join(".gitignore"), GITIGNORE).context("Failed to write .gitignore")?;
    Ok(())
}

/// Initializes a Git repository unless the project is already inside one.
///
/// Nix only applies `.gitignore` filtering to path flakes inside a Git
/// worktree. Without this, `nix develop` copies the entire project, including
/// multi-gigabyte build outputs, into the store.
pub fn init_git(project_dir: &Path) -> Result<()> {
    let already_inside = git_says_inside_worktree(project_dir);

    if !already_inside {
        let status = Command::new("git")
            .arg("init")
            .arg("-q")
            .current_dir(project_dir)
            .status();

        match status {
            Ok(status) if status.success() => {}
            Ok(status) => bail!("git init failed with status {status}"),
            Err(error) => {
                println!("Git not available; skipping repository initialization: {error}");
                return Ok(());
            }
        }
    }

    // Nix only sees tracked/staged files in a Git source. Stage the generated
    // files so the flake works without requiring an immediate commit.
    match Command::new("git")
        .args(["add", "."])
        .current_dir(project_dir)
        .status()
    {
        Ok(status) if status.success() => {
            if already_inside {
                println!("Staged generated project files with Git.");
            } else {
                println!("Initialized Git repository and staged generated files.");
            }
        }
        Ok(status) => println!("Warning: git add failed with status {status}"),
        Err(error) => println!("Warning: could not stage generated files: {error}"),
    }

    Ok(())
}

fn git_says_inside_worktree(project_dir: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(project_dir)
        .output()
        .map(|output| {
            output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "true"
        })
        .unwrap_or(false)
}

fn write_all_in_one_assets(project_dir: &Path) -> Result<()> {
    fs::write(
        project_dir.join("src").join("custom_shader.wgsl"),
        ALL_IN_ONE_CUSTOM_SHADER,
    )
    .context("Failed to write src/custom_shader.wgsl")?;

    let assets = project_dir.join("Assets");

    fs::create_dir_all(assets.join("Scenes")).context("Failed to create Assets/Scenes")?;
    fs::write(
        assets.join("Scenes").join("procgeo_demo.ron"),
        ALL_IN_ONE_SCENE,
    )
    .context("Failed to write Assets/Scenes/procgeo_demo.ron")?;

    fs::create_dir_all(assets.join("Models")).context("Failed to create Assets/Models")?;
    fs::write(
        assets.join("Models").join("DamagedHelmet.glb"),
        ALL_IN_ONE_HELMET,
    )
    .context("Failed to write Assets/Models/DamagedHelmet.glb")?;

    Ok(())
}

fn generate_cargo_toml(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let lib_name = config.project_name.replace('-', "_").to_lowercase();

    let orbital_dep = if let Some(path) = &config.engine_path {
        format!(r#"orbital = {{ path = "{path}/Crates/orbital" }}"#)
    } else {
        format!(
            r#"orbital = {{ git = "{repo}", branch = "{branch}" }}"#,
            repo = config.engine_repo,
            branch = config.engine_branch,
        )
    };

    let content = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "{name}_desktop"
path = "src/main.rs"

[lib]
name = "{lib_name}"
crate-type = ["cdylib", "lib"]

[dependencies]
{orbital_dep}

# The engine and cryoglyph both use the same patched cosmic-text fork.
# This patch pins any crates.io consumer of cosmic-text to that revision,
# so cargo cannot resolve two incompatible cosmic-text versions (which
# makes iced_wgpu fail to compile).
[patch.crates-io]
cosmic-text = {{ git = "https://github.com/hecrj/cosmic-text.git", rev = "19cca2bd90e73e20614964f06d44240e71e503b3" }}
"#,
        name = config.project_name,
        lib_name = lib_name,
        orbital_dep = orbital_dep,
    );

    fs::write(project_dir.join("Cargo.toml"), content).context("Failed to write Cargo.toml")?;

    Ok(())
}

fn generate_orbital_toml(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let content = format!(
        r#"[orbital]

[android]
package = "{package}"
min_sdk = {min_sdk}
target_sdk = {target_sdk}
targets = ["arm64-v8a", "armeabi-v7a", "x86_64", "x86"]
apk_mode = "multiarch"
ndk_version = "26.2.11394342"
screen_orientation = "unspecified"

# Signing configuration for release APKs (optional)
# If not specified, a debug keystore will be automatically generated
# keystore_path = "keystore/release.keystore"
# keystore_password = "your_keystore_password"
# key_alias = "your_key_alias"
# key_password = "your_key_password"
"#,
        package = config.package_name,
        min_sdk = config.min_sdk,
        target_sdk = config.target_sdk,
    );

    fs::write(project_dir.join("Orbital.toml"), content).context("Failed to write Orbital.toml")?;

    Ok(())
}

/// Generates the direnv + flake development environment for a project.
///
/// The generated environment forwards the engine flake's dev shell, which
/// provides the Rust toolchain, the engine's system dependencies, and the
/// `orbital` CLI (installed on shell entry, refreshable with `orbital-update`).
/// This works without a local engine checkout: Nix fetches the engine flake
/// from the configured git repository.
fn generate_devenv(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let (envrc, flake) = if let Some(path) = &config.engine_path {
        // Local engine checkout: canonicalize so the generated files stay
        // valid regardless of where the project is created.
        let engine_path = fs::canonicalize(path)
            .with_context(|| format!("Failed to resolve engine path: {path}"))?;
        let engine_path_str = engine_path.to_string_lossy();

        // Prefer Git over a raw path input: `path:` copies the entire engine
        // checkout (including target/) into the store, while `git+file:`
        // applies Git filtering and includes uncommitted worktree changes.
        let orbital_url = if engine_path.join(".git").exists() {
            format!("git+file://{engine_path_str}")
        } else {
            format!("path:{engine_path_str}")
        };

        (
            format!(
                "# Generated by `orbital init`. Refresh the CLI with `orbital-update`.\n\
                 # For Android builds, run `nix develop .#android` (Android SDK/JDK stay opt-in).\n\
                 # For the Android emulator, run `nix develop .#android-emulator` (larger download).\n\
                 export ORBITAL_CLI_PATH=\"{engine_path_str}/Crates/orbital_cli\"\n\
                 use flake\n"
            ),
            devenv_flake(&orbital_url),
        )
    } else {
        (
            format!(
                "# Generated by `orbital init`. Refresh the CLI with `orbital-update`.\n\
                 # For Android builds, run `nix develop .#android` (Android SDK/JDK stay opt-in).\n\
                 # For the Android emulator, run `nix develop .#android-emulator` (larger download).\n\
                 export ORBITAL_CLI_REPO=\"{repo}\"\n\
                 export ORBITAL_CLI_BRANCH=\"{branch}\"\n\
                 use flake\n",
                repo = config.engine_repo,
                branch = config.engine_branch,
            ),
            devenv_flake_with_project_android(&format!(
                "git+{repo}?ref={branch}",
                repo = config.engine_repo,
                branch = config.engine_branch,
            )),
        )
    };

    if config.engine_path.is_none() {
        // Remote engine: the generated project owns its Android dev shells so
        // it does not depend on the engine flake shipping an Android env.
        fs::create_dir_all(project_dir.join("nix")).context("Failed to create nix directory")?;
        fs::write(
            project_dir.join("nix").join("android.nix"),
            ANDROID_NIX_TEMPLATE,
        )
        .context("Failed to write nix/android.nix")?;
    }

    fs::write(project_dir.join(".envrc"), envrc).context("Failed to write .envrc")?;
    fs::write(project_dir.join("flake.nix"), flake).context("Failed to write flake.nix")?;

    Ok(())
}

/// Builds a `flake.nix` that re-exports the engine flake's default dev shell.
fn devenv_flake(orbital_url: &str) -> String {
    format!(
        r#"# Generated by `orbital init`.
#
# The development environment (Rust toolchain, engine system dependencies and
# the `orbital` CLI) is provided by the Orbital engine flake. Update the engine
# with `nix flake update orbital`, then rebuild the CLI with `orbital-update`.
{{
  description = "Orbital project development environment";

  inputs.orbital.url = "{orbital_url}";

  outputs = {{ orbital, ... }}:
    {{
      # Re-export every engine dev shell so opt-in shells such as
      # `nix develop .#android` remain available in generated projects.
      devShells = orbital.devShells;
    }};
}}
"#
    )
}

/// Builds a `flake.nix` for projects without a local engine checkout.
///
/// The default shell still comes from the engine flake, but the Android shells
/// are generated into the project so the engine does not have to provide an
/// Android environment for standalone projects.
fn devenv_flake_with_project_android(orbital_url: &str) -> String {
    format!(
        r#"# Generated by `orbital init`.
#
# The default development environment comes from the Orbital engine flake.
# Android shells are defined locally in ./nix/android.nix.
#
# Update the engine with `nix flake update orbital`, then rebuild the CLI with
# `orbital-update`.
{{
  description = "Orbital project development environment";

  inputs = {{
    orbital.url = "{orbital_url}";
    nixpkgs.follows = "orbital/nixpkgs";
    flake-utils.follows = "orbital/flake-utils";
  }};

  outputs = {{ self, nixpkgs, flake-utils, orbital, ... }}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${{system}};
        pkgsAndroid = import nixpkgs {{
          inherit system;
          config = {{
            allowUnfree = true;
            android_sdk.accept_license = true;
          }};
        }};
        androidShells = import ./nix/android.nix {{
          inherit pkgs pkgsAndroid;
          rustToolchainAndroid = orbital.packages.${{system}}.rustToolchainAndroid;
        }};
      in
      {{
        devShells = orbital.devShells.${{system}} // androidShells;
      }});
}}
"#
    )
}

fn generate_lib_rs(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let content = template_lib_source(&config.template)?;

    fs::write(project_dir.join("src").join("lib.rs"), content)
        .context("Failed to write src/lib.rs")?;

    Ok(())
}

/// Returns the embedded application source for a template.
fn template_lib_source(template: &str) -> Result<&'static str> {
    match template {
        "minimal" => Ok(MINIMAL_TEMPLATE),
        "all-in-one" => Ok(ALL_IN_ONE_TEMPLATE),
        "2d" => Ok(TWO_D_TEMPLATE),
        other => bail!("Unknown template '{other}'. Available templates: minimal, all-in-one, 2d"),
    }
}

fn generate_main_rs(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let lib_name = config.project_name.replace('-', "_").to_lowercase();

    let content = format!(
        r#"fn main() {{
    // Desktop entry point - `run()` is generated by `#[orbital::main]`.
    {lib_name}::run();
}}
"#,
        lib_name = lib_name,
    );

    fs::write(project_dir.join("src").join("main.rs"), content)
        .context("Failed to write src/main.rs")?;

    Ok(())
}

// The canonical template sources are workspace examples. Embedding them keeps
// `orbital init` self-contained when the CLI is installed from git or a path.
const MINIMAL_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../Examples/minimal/src/lib.rs"
));

const ALL_IN_ONE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../Examples/all-in-one/src/lib.rs"
));

const ALL_IN_ONE_SCENE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../Examples/all-in-one/Assets/Scenes/procgeo_demo.ron"
));

const ALL_IN_ONE_HELMET: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../Examples/all-in-one/Assets/Models/DamagedHelmet.glb"
));

const ALL_IN_ONE_CUSTOM_SHADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../Examples/all-in-one/src/custom_shader.wgsl"
));

const TWO_D_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../Examples/2d/src/lib.rs"
));

const ANDROID_NIX_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/template/android.nix"
));
