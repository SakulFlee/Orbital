#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CARGO_TARGET_ROOT="${CARGO_TARGET_DIR:-$ROOT/target}"
CLI="$CARGO_TARGET_ROOT/debug/orbital"
TEMPLATE_TARGET_DIR="$CARGO_TARGET_ROOT/template-check"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/orbital-templates.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

export SKIP_GLTF_EXPORT=1

echo "Building the orbital CLI..."
cargo build --manifest-path "$ROOT/Cargo.toml" -p orbital-cli

for template in minimal all-in-one 2d; do
    project="$TMP/$template"

    echo "Checking template '$template'..."
    "$CLI" init "template-check-$template" \
        --yes \
        --template "$template" \
        --project-path "$project" \
        --engine-path "$ROOT"

    for file in Cargo.toml Orbital.toml .envrc flake.nix src/lib.rs src/main.rs Assets/.gitkeep; do
        if [[ ! -e "$project/$file" ]]; then
            echo "Missing generated file: $project/$file" >&2
            exit 1
        fi
    done

    if grep -Rq "{{PROJECT_NAME}}" "$project/src"; then
        echo "Unreplaced {{PROJECT_NAME}} placeholder in $project/src" >&2
        exit 1
    fi

    # Reuse the workspace lockfile so dependency resolution does not drift
    # between the engine and the generated project.
    cp "$ROOT/Cargo.lock" "$project/Cargo.lock"

    (
        cd "$project"
        CARGO_TARGET_DIR="$TEMPLATE_TARGET_DIR" cargo check
    )
done

echo "All templates generated and checked successfully."
