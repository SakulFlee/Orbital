use anyhow::{Context, Result};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config;

pub const SHELL_ANDROID: &str = "android";
pub const SHELL_ANDROID_EMULATOR: &str = "android-emulator";

const ANDROID_SHELL_ENV: &str = "ORBITAL_ANDROID_SHELL";
const DISABLE_ENV: &str = "ORBITAL_NO_NIX_ANDROID_SHELL";
const SHELL_REEXEC_GUARD: &str = "ORBITAL_NIX_ANDROID_REEXEC";
const EMULATOR_REEXEC_GUARD: &str = "ORBITAL_NIX_ANDROID_EMULATOR_REEXEC";
const SKIP_CLI_INSTALL_ENV: &str = "ORBITAL_SKIP_CLI_INSTALL";

/// Re-exec an Android command inside `nix develop .#android` when the project
/// is Nix-managed and we are not already in an Android environment.
pub fn maybe_enter_android_shell(is_android_command: bool) -> Result<()> {
    if !is_android_command || is_disabled() || std::env::var_os(SHELL_REEXEC_GUARD).is_some() {
        return Ok(());
    }

    if active_android_shell().is_some() || android_home_is_set() {
        return Ok(());
    }

    let Some(project_root) = nix_project_root() else {
        return Ok(());
    };

    println!("Nix project detected; entering the Android dev shell (`nix develop .#android`).");
    reexec(
        &project_root,
        SHELL_ANDROID,
        &[(SHELL_REEXEC_GUARD, "1"), (SKIP_CLI_INSTALL_ENV, "1")],
    )
}

/// Whether the current process can retry inside the emulator-enabled shell.
pub fn can_enter_android_emulator_shell() -> bool {
    !is_disabled()
        && std::env::var_os(EMULATOR_REEXEC_GUARD).is_none()
        && active_android_shell().as_deref() == Some(SHELL_ANDROID)
        && nix_project_root().is_some()
}

/// Re-exec the current command inside `nix develop .#android-emulator`.
pub fn reexec_in_android_emulator_shell() -> Result<()> {
    let Some(project_root) = nix_project_root() else {
        anyhow::bail!(
            "Cannot enter the Android emulator shell without an Orbital project containing flake.nix."
        );
    };

    println!(
        "Entering the Android emulator dev shell (`nix develop .#android-emulator`); first setup downloads the emulator and system image (~2 GB)."
    );
    reexec(
        &project_root,
        SHELL_ANDROID_EMULATOR,
        &[
            (SHELL_REEXEC_GUARD, "1"),
            (EMULATOR_REEXEC_GUARD, "1"),
            (SKIP_CLI_INSTALL_ENV, "1"),
        ],
    )
}

fn is_disabled() -> bool {
    std::env::var_os(DISABLE_ENV).is_some()
}

fn active_android_shell() -> Option<String> {
    std::env::var(ANDROID_SHELL_ENV)
        .ok()
        .filter(|value| !value.is_empty())
}

fn android_home_is_set() -> bool {
    std::env::var_os("ANDROID_HOME")
        .map(PathBuf::from)
        .is_some_and(|path| path.exists())
}

fn nix_project_root() -> Option<PathBuf> {
    let root = config::find_project_root().ok()?;
    if !root.join("flake.nix").is_file() || !nix_available() {
        return None;
    }
    Some(root)
}

fn nix_available() -> bool {
    Command::new("nix")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn reexec(project_root: &Path, shell: &str, extra_env: &[(&str, &str)]) -> Result<()> {
    let executable = std::env::current_exe().context("Failed to locate the orbital executable")?;
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();

    let profile_dir = project_root.join(".direnv");
    let profile_path = profile_dir.join(format!("{shell}-profile"));
    let use_profile = std::fs::create_dir_all(&profile_dir).is_ok();

    let mut command = Command::new("nix");
    command.arg("develop").arg(format!(".#{shell}"));
    if use_profile {
        command.arg("--profile").arg(&profile_path);
    }
    command
        .arg("--command")
        .arg(&executable)
        .args(&args)
        .current_dir(project_root);

    for (key, value) in extra_env {
        command.env(key, value);
    }

    let status = command
        .status()
        .with_context(|| format!("Failed to run `nix develop .#{shell}`"))?;

    std::process::exit(status.code().unwrap_or(1));
}
