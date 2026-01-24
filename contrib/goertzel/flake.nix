{
  description = "Python development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (pkgs.python3.withPackages (python-pkgs: [
                python-pkgs.tkinter
                python-pkgs.notebook
                python-pkgs.ipympl
                python-pkgs.numpy
                python-pkgs.scipy
                python-pkgs.soundfile
                python-pkgs.matplotlib
                python-pkgs.hypothesis
            ]))
          ];
        };
      }
    );
}

