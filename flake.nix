{
  description = "Simple Rust development environment with Fenix Nightly";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        rustToolchain = fenix.packages.${system}.latest.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
          "rust-analyzer"
        ];

        rustToolchainCross = fenix.packages.${system}.combine [
          rustToolchain
          fenix.packages.${system}.targets."aarch64-unknown-linux-gnu".latest.rust-std
          fenix.packages.${system}.targets."x86_64-pc-windows-gnu".latest.rust-std
        ];

        commonTools = with pkgs; [
          pkgs.gcc
          pkgs.binutils
          pkgs.gnumake
          pkgs.git
          pkgs.cargo-flamegraph
          pkgs.cargo-criterion
          pkgs.pkg-config
          pkgs.blender
        ];

        commonLinuxBuildInputs = with pkgs; [
          pkgs.glibc.dev
          pkgs.systemd
          wayland
          libxkbcommon
          vulkan-loader
          pkgs.mesa
        ];

        commonLinuxLdLibraryPath = pkgs.lib.makeLibraryPath [
          pkgs.wayland
          pkgs.libxkbcommon
          pkgs.vulkan-loader
          pkgs.mesa
        ];

        dummyLibpthread = pkgs.runCommand "dummy-libpthread-x86_64-w64-mingw32" {} ''
          mkdir -p $out/lib
          ${pkgs.binutils}/bin/ar rcs $out/lib/libpthread.a
        '';
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [ rustToolchain ] ++ commonTools;

          buildInputs = commonLinuxBuildInputs;

          shellHook = ''
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${commonLinuxLdLibraryPath}"
            mkdir -p .cargo/bin
            ln -sf "$(which gcc)" .cargo/bin/x86_64-linux-gnu-gcc
            export PATH="$PWD/.cargo/bin:$PATH"
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$(which gcc)"

            # Keep a project-local `orbital` CLI on PATH. It is installed into
            # .cargo/bin (already prepended to PATH above) so it survives direnv
            # reloads and never requires a flake rebuild.
            #
            # Source of truth, in order of preference:
            #   1. ORBITAL_CLI_PATH            (generated project, local engine)
            #   2. Crates/orbital_cli in $PWD  (this engine checkout)
            #   3. Forgejo git repository       (generated project, remote engine)
            _orbital_install_cli() {
              if [ -n "''${ORBITAL_CLI_PATH:-}" ]; then
                cargo install --path "$ORBITAL_CLI_PATH" --root "$PWD/.cargo" --debug --locked --force
              elif [ -f "$PWD/Crates/orbital_cli/Cargo.toml" ]; then
                cargo install --path "$PWD/Crates/orbital_cli" --root "$PWD/.cargo" --debug --locked --force
              else
                cargo install --git "''${ORBITAL_CLI_REPO:-https://forgejo.sakul-flee.de/SakulFlee/Orbital.git}" \
                  --branch "''${ORBITAL_CLI_BRANCH:-main}" \
                  --root "$PWD/.cargo" --debug --force orbital-cli
              fi
            }
            orbital-update() { _orbital_install_cli; }

            if [ -f "$PWD/Crates/orbital_cli/Cargo.toml" ]; then
              # Engine checkout: rebuild only when the CLI sources or the
              # workspace lockfile are newer than the installed binary.
              if [ ! -x "$PWD/.cargo/bin/orbital" ] || \
                 [ -n "$(find "$PWD/Crates/orbital_cli/src" "$PWD/Crates/orbital_cli/Cargo.toml" "$PWD/Cargo.lock" -newer "$PWD/.cargo/bin/orbital" -print -quit 2>/dev/null)" ]; then
                echo "[direnv] Building orbital CLI..."
                _orbital_install_cli
              fi
            elif [ ! -x "$PWD/.cargo/bin/orbital" ]; then
              # Generated project: install once, refresh with `orbital-update`.
              echo "[direnv] Installing orbital CLI..."
              _orbital_install_cli
            fi
          '';
        };

        devShells.aarch64-cross = pkgs.mkShell {
          nativeBuildInputs = [ rustToolchainCross ] ++ commonTools ++ [
            pkgs.pkgsCross.aarch64-multiplatform.buildPackages.stdenv.cc
            pkgs.pkgsCross.aarch64-multiplatform.buildPackages.pkg-config
          ];

          buildInputs = with pkgs.pkgsCross.aarch64-multiplatform; [
            glibc.dev
            systemd
            wayland
            libxkbcommon
            vulkan-loader
          ];

          shellHook = ''
            export RUST_SRC_PATH="${rustToolchainCross}/lib/rustlib/src/rust/library"
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${commonLinuxLdLibraryPath}"
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="${pkgs.pkgsCross.aarch64-multiplatform.stdenv.cc}/bin/aarch64-unknown-linux-gnu-cc"
            export CC_aarch64_unknown_linux_gnu="${pkgs.pkgsCross.aarch64-multiplatform.stdenv.cc}/bin/aarch64-unknown-linux-gnu-cc"
            export PKG_CONFIG_ALLOW_CROSS=1
          '';
        };

        devShells.windows-x86_64-cross = pkgs.mkShell {
          nativeBuildInputs = [ rustToolchainCross ] ++ commonTools ++ [
            pkgs.pkgsCross.mingwW64.buildPackages.stdenv.cc
            pkgs.pkgsCross.mingwW64.buildPackages.pkg-config
            pkgs.pkgsCross.mingwW64.buildPackages.binutils
            dummyLibpthread
          ];

          shellHook = ''
            export RUST_SRC_PATH="${rustToolchainCross}/lib/rustlib/src/rust/library"
            export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/x86_64-w64-mingw32-gcc"
            export CC_x86_64_pc_windows_gnu="${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/x86_64-w64-mingw32-gcc"
            export CXX_x86_64_pc_windows_gnu="${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/x86_64-w64-mingw32-g++"
            export AR_x86_64_pc_windows_gnu="${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/x86_64-w64-mingw32-ar"
            export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-L${dummyLibpthread}/lib"
            export PKG_CONFIG_ALLOW_CROSS=1
          '';
        };
      });
}
