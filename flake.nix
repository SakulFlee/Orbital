{
  description = "Rust development environment using fenix";

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
        
        # Change "stable" to "latest" or "complete" if needed
        rustToolchain = fenix.packages.${system}.nightly.withComponents [
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
          # Native build inputs (tools needed at compile time)
          nativeBuildInputs = [
            rustToolchain
            pkgs.pkg-config
            pkgs.blender
            pkgs.cargo-flamegraph
            pkgs.cargo-criterion
          ];

          # Build inputs (libraries your app links against)
          buildInputs = with pkgs; [
            openssl
          ];

          # Environment variables
          shellHook = ''
            # Ensures rust-analyzer can find the standard library source
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
          '';
        };
      }
    );
}
