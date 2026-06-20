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

        # latest = nightly
        # stable = stable
        rustToolchain = fenix.packages.${system}.latest.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
          "rust-analyzer"
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            # Fenix toolchain
            rustToolchain
            
            # Additional Rust tools
            pkgs.cargo-flamegraph
            pkgs.cargo-criterion

            # Build Dependencies
            pkgs.pkg-config
            pkgs.blender
          ];

          # Runtime dependencies
          buildInputs = [
          
          ];

          shellHook = ''
            # 4. Point rust-analyzer directly to the fenix toolchain standard library source
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
          '';
        };
      });
}
