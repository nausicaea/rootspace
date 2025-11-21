let
  pkgs = import (builtins.fetchTarball { url =  https://github.com/NixOS/nixpkgs/archive/1af9d0d88739a3e0b0e64add12adce320ba1163f.tar.gz; sha256 = "1hka7jk0hxdkp859hxx7qwzkwwq69mazbxm4zc3nm8lch9h2clr6"; }) {};
in

pkgs.mkShell {
  buildInputs = [
    pkgs.cacert
    pkgs.git
    pkgs.pre-commit
    pkgs.cargo
    pkgs.rust-analyzer
    pkgs.cargo-nextest
    pkgs.cargo-fuzz
    pkgs.cargo-audit
    pkgs.cargo-auditable
    pkgs.tokio-console
    pkgs.rusty-man
  ];
  RUST_LOG = "warn,rootspace=trace,griffon=trace";
  #RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

}
