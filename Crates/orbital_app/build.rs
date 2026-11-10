use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let repo = manifest
        .parent()
        .and_then(|crates| crates.parent())
        .unwrap_or(&manifest);

    let revision = std::env::var("ORBITAL_GIT_REVISION")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            Command::new("git")
                .arg("-C")
                .arg(repo)
                .args(["rev-parse", "--short=12", "HEAD"])
                .output()
                .ok()
                .filter(|output| output.status.success())
                .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=ORBITAL_GIT_REVISION={revision}");
    println!("cargo:rerun-if-env-changed=ORBITAL_GIT_REVISION");

    let head = repo.join(".git/HEAD");
    if head.is_file() {
        println!("cargo:rerun-if-changed={}", head.display());
    }
}
