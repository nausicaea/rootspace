{ pkgs ? import (fetchTarball https://github.com/NixOS/nixpkgs/archive/1af9d0d88739a3e0b0e64add12adce320ba1163f.tar.gz) {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.cacert
    pkgs.git
    pkgs.pre-commit
    pkgs.cargo
    pkgs.cargo-nextest
    pkgs.cargo-fuzz
    pkgs.cargo-audit
    pkgs.cargo-auditable
    pkgs.tokio-console
  ];
  RUST_LOG = "warn,rootspace=trace,griffon=trace";
}
