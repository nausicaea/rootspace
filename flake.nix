{
  description = "Rust stable development environment";

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
          config.allowUnfree = true;
        };

        rustEnv = pkgs.rust-bin.stable."1.91.1".default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustEnv
          ];

          buildInputs = with pkgs; [
            openssl
            pkg-config
            git
            trufflehog
            gitleaks
            pre-commit
            rust-analyzer
            cargo-nextest
            cargo-audit
            cargo-auditable
            cargo-machete
            cargo-sweep
            bacon
            rusty-man
            jetbrains.rust-rover
          ];

          shellHook = ''
            mkdir -p ~/.rust-rover/toolchain

            ln -sfn ${rustEnv}/lib ~/.rust-rover/toolchain
            ln -sfn ${rustEnv}/bin ~/.rust-rover/toolchain

            export RUST_SRC_PATH="$HOME/.rust-rover/toolchain/lib/rustlib/src/rust/library"
            export RUST_LOG="warn,rootspace=trace,griffon=info,glamour=trace"
          '';
        };
      }
    );
}
