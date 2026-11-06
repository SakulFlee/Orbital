#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn android_build_reexecs_in_nix_android_shell() {
    let root = temp_dir();
    fs::write(root.join("Orbital.toml"), "[orbital]\n").unwrap();
    fs::write(root.join("flake.nix"), "{}\n").unwrap();

    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let log = root.join("nix.log");
    let nix = bin.join("nix");
    fs::write(
        &nix,
        r#"#!/bin/sh
{
  printf 'CALL\n'
  for arg in "$@"; do printf 'ARG=%s\n' "$arg"; done
  printf 'GUARD=%s\n' "${ORBITAL_NIX_ANDROID_REEXEC:-}"
  printf 'SKIP=%s\n' "${ORBITAL_SKIP_CLI_INSTALL:-}"
} >> "$ORBITAL_TEST_NIX_LOG"
exit 0
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&nix).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&nix, permissions).unwrap();

    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new(env!("CARGO_BIN_EXE_orbital"))
        .args(["build", "android"])
        .current_dir(&root)
        .env("PATH", path)
        .env("ORBITAL_TEST_NIX_LOG", &log)
        .env_remove("ANDROID_HOME")
        .env_remove("ORBITAL_ANDROID_SHELL")
        .env_remove("ORBITAL_NIX_ANDROID_REEXEC")
        .env_remove("ORBITAL_NIX_ANDROID_EMULATOR_REEXEC")
        .env_remove("ORBITAL_NO_NIX_ANDROID_SHELL")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "orbital failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fs::read_to_string(&log).unwrap();
    for expected in [
        "ARG=develop",
        "ARG=.#android",
        "ARG=--profile",
        ".direnv/android-profile",
        "ARG=--command",
        "ARG=build",
        "ARG=android",
        "GUARD=1",
        "SKIP=1",
    ] {
        assert!(log.contains(expected), "missing {expected:?} in:\n{log}");
    }

    fs::remove_dir_all(root).unwrap();
}

fn temp_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "orbital-cli-nix-reexec-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}
