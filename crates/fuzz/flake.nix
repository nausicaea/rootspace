{
  description = "Rust nightly development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustEnv = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" "llvm-tools" ];
          targets = [ "aarch64-apple-darwin" ];
        });
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustEnv
          ];

          buildInputs = with pkgs; [
            openssl
            pkg-config
            git
            cargo-fuzz
            #llvmPackages.bintools
            cargo-audit
            cargo-auditable
            cargo-machete
            cargo-sweep
            bacon
          ];
        };
      }
    );
}
