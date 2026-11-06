#!/usr/bin/env bash
# Build and package every runnable example for a release.
#
# Usage:
#   package-examples.sh desktop <rust-target>
#   package-examples.sh android <rust-target> <android-abi>
#   package-examples.sh ios <rust-target>
#
# Desktop packages contain the example executable and an Assets/ tree.
# Android packages contain the example cdylib under jniLibs/<abi>/ plus the
# same Assets/ tree, ready to drop into a generated Android project.
# iOS packages contain the static library and Assets/ for an Xcode project.
set -euo pipefail

MODE="${1:?usage: package-examples.sh <desktop|android|ios> <target> [abi]}"
TARGET="${2:?usage: package-examples.sh <desktop|android|ios> <target> [abi]}"
ABI="${3:-}"
PROFILE="${ORBITAL_PROFILE:-release}"
# Optional local-use filter; CI always packages every example.
EXAMPLE_FILTER="${ORBITAL_EXAMPLE_FILTER:-}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST="$ROOT/dist/examples"
PROFILE_DIR="$PROFILE"
if [[ "$PROFILE" == "dev" ]]; then
    PROFILE_DIR="debug"
fi
OUT="$ROOT/target/$TARGET/$PROFILE_DIR"
mkdir -p "$DIST"

if [[ "$MODE" == "android" && -z "$ABI" ]]; then
    echo "Android packaging needs an ABI (for example arm64-v8a)" >&2
    exit 1
fi

for manifest in "$ROOT"/Examples/*/Cargo.toml; do
    example_dir="$(dirname "$manifest")"
    package="$(sed -n 's/^name = "\(.*\)"/\1/p' "$manifest" | head -n 1)"

    if [[ -z "$package" ]]; then
        echo "Could not read package name from $manifest" >&2
        exit 1
    fi

    if [[ -n "$EXAMPLE_FILTER" && "$package" != "$EXAMPLE_FILTER" ]]; then
        continue
    fi

    echo "Packaging example '$package' for $MODE/$TARGET"
    stage="$DIST/$package-$TARGET"
    rm -rf "$stage"
    mkdir -p "$stage/Assets"

    case "$MODE" in
        desktop)
            if [[ "${ORBITAL_SKIP_BUILD:-0}" != "1" ]]; then
                cargo build --profile "$PROFILE" --target "$TARGET" -p "$package" --bin "$package"
            fi
            executable="$OUT/$package"
            if [[ ! -f "$executable" && -f "$executable.exe" ]]; then
                executable="$executable.exe"
            fi
            if [[ ! -f "$executable" ]]; then
                echo "Missing built executable for $package" >&2
                exit 1
            fi
            cp "$executable" "$stage/"
            cat >"$stage/README.txt" <<EOF
Run ./$(basename "$executable") from this directory.
The Assets/ tree next to the executable is required at runtime.
EOF
            ;;
        android)
            if [[ "${ORBITAL_SKIP_BUILD:-0}" != "1" ]]; then
                cargo ndk -t "$TARGET" -o "$stage/jniLibs" build --profile "$PROFILE" --lib -p "$package"
            fi
            if ! find "$stage/jniLibs" -type f -name "lib$package.so" | grep -q .; then
                echo "Missing Android cdylib for $package" >&2
                exit 1
            fi
            cat >"$stage/README.txt" <<EOF
Copy jniLibs/$ABI/lib$package.so into the generated Android project's
app/src/main/jniLibs/$ABI/ directory and copy Assets/ to the project root.
EOF
            ;;
        ios)
            if [[ "${ORBITAL_SKIP_BUILD:-0}" != "1" ]]; then
                cargo build --profile "$PROFILE" --target "$TARGET" -p "$package" --lib
            fi
            library="$OUT/lib$package.a"
            if [[ ! -f "$library" ]]; then
                # Example crates expose a cdylib; the iOS template links the
                # generated staticlib. Accept either so this packaging stays
                # useful for both layouts.
                library="$OUT/lib$package.dylib"
            fi
            if [[ ! -f "$library" ]]; then
                echo "Missing iOS library for $package" >&2
                exit 1
            fi
            cp "$library" "$stage/"
            cat >"$stage/README.txt" <<EOF
Link $(basename "$library") into the Xcode app and copy Assets/ to the
app bundle resources.
EOF
            ;;
        *)
            echo "Unknown mode: $MODE" >&2
            exit 1
            ;;
    esac

    # The engine's shared assets are required by several examples; overlay the
    # example's own Assets/ on top so its scene and model files win.
    cp -a "$ROOT/Assets/." "$stage/Assets/"
    if [[ -d "$example_dir/Assets" ]]; then
        cp -a "$example_dir/Assets/." "$stage/Assets/"
    fi

    archive="$DIST/$package-$TARGET"
    if [[ "$MODE" == "android" ]]; then
        archive="$archive-$ABI"
    fi
    rm -f "$archive.zip"
    (
        cd "$DIST"
        zip -qr "$(basename "$archive").zip" "$(basename "$stage")"
    )
    rm -rf "$stage"
done
