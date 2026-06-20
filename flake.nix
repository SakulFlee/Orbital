{
  description = "Simple Rust development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            # Rust tools
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
            
            # Additional Rust tools
            cargo-flamegraph
            cargo-criterion

            # Dependencies
            pkg-config
            blender
          ];

          buildInputs = with pkgs; [
          ];

          shellHook = ''
            # Nixpkgs provides standard library sources in rustPlatform
            export RUST_SRC_PATH="${pkgs.rustPlatform.rustLibSrc}"
          '';
        };
      });
}
