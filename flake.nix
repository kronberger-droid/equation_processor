{
  description = "Rust development shell with Fenix and tectonic for document rendering";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url  = "github:numtide/flake-utils";
    fenix.url        = "github:nix-community/fenix";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, fenix, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default rust-overlay.overlays.default ];
        };
        lib = pkgs.lib;

        # Fenix-provided stable Rust and analyzer
        stableToolchain = fenix.packages.${system}.complete.toolchain;
        rustAnalyzer    = fenix.packages.${system}.latest.rust-analyzer;
      in {
        devShells.default = pkgs.mkShell {
          name = "rust-dev-shell";

          buildInputs = with pkgs; lib.flatten [
            stableToolchain
            rustAnalyzer
            cargo-expand    # Inspect expanded macros
            nushell         # Friendly REPL shell
            tectonic        # LaTeX to PDF
            poppler_utils   # PDF utilities (pdftocairo, etc.)
          ];

          shellHook = ''
            echo "Using Rust toolchain: $(rustc --version)"
            # Isolate Cargo and Rustup directories in your home
            export CARGO_HOME="$HOME/.cargo"
            export RUSTUP_HOME="$HOME/.rustup"
            mkdir -p "$CARGO_HOME" "$RUSTUP_HOME"

            # Launch nushell as login shell
            exec nu --login
          '';
        };
      }
    );
}
