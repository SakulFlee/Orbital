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

        commonNativeBuildInputs = with pkgs; [
          rustToolchain
          pkgs.gcc
          pkgs.binutils
          pkgs.gnumake
          pkgs.cargo-flamegraph
          pkgs.cargo-criterion
          pkgs.pkg-config
          pkgs.blender
        ];

        commonBuildInputs = with pkgs; [
          pkgs.glibc.dev
          pkgs.systemd
          wayland
          libxkbcommon
          vulkan-loader
          pkgs.mesa
        ];

        commonShellHook = ''
          export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [ pkgs.wayland pkgs.libxkbcommon pkgs.vulkan-loader pkgs.mesa ]}"
        '';
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = commonNativeBuildInputs;

          buildInputs = commonBuildInputs;

          shellHook = commonShellHook + ''
            mkdir -p .cargo/bin
            ln -sf "$(which gcc)" .cargo/bin/x86_64-linux-gnu-gcc
            export PATH="$PWD/.cargo/bin:$PATH"
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$(which gcc)"
          '';
        };

        devShells.aarch64-cross = pkgs.mkShell {
          nativeBuildInputs = commonNativeBuildInputs ++ [
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

          shellHook = commonShellHook + ''
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="${pkgs.pkgsCross.aarch64-multiplatform.stdenv.cc}/bin/aarch64-unknown-linux-gnu-cc"
            export CC_aarch64_unknown_linux_gnu="${pkgs.pkgsCross.aarch64-multiplatform.stdenv.cc}/bin/aarch64-unknown-linux-gnu-cc"
            export PKG_CONFIG_ALLOW_CROSS=1
          '';
        };
      });
}
