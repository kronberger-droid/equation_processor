{ description = "Basic flake";

inputs = {
  nixpkgs.url     = "github:NixOS/nixpkgs/nixos-unstable";
  flake-utils.url = "github:numtide/flake-utils";
};

outputs = {nixpkgs, flake-utils, ... }:
  flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells = {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # rust development
            rustup
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer

            # tools
            nushell
            cargo-expand
            tectonic
            poppler_utils
          ];

          shellHook = ''
            # rust toolchain init
            if ! rustup toolchain list | grep -q stable; then
              rustup toolchain install stable
            fi
            rustup default stable

            # start into nushell
            nu
          '';
        };
      };
    }
  );
}
