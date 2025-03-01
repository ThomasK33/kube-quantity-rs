{
  description = "kube_quantity - Rust Kubernetes Quantity Parser Library";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        formatter = pkgs.nixfmt-rfc-style;

        craneLib = crane.mkLib pkgs;
        markdownFilter = path: _type: builtins.match ".*md$" path != null;
        markdownOrCargo = path: type: (markdownFilter path type) || (craneLib.filterCargoSources path type);
        # Common arguments for all crane builds
        commonArgs = {
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = markdownOrCargo;
            name = "source"; # Be reproducible, regardless of the directory name
          };
          strictDeps = true;

          cargoExtraArgs = "--features __check";
        };
        # Build just the cargo dependencies
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in
      {
        inherit formatter;

        checks = {
          # Check code format
          cargo-fmt = craneLib.cargoFmt (builtins.removeAttrs commonArgs [ "cargoExtraArgs" ]);
          # Run clippy (with the dependencies already built)
          cargo-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
          # Run tests
          cargo-test = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
        };

        devShells = {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo
              formatter
              rustc
              clippy
            ];
          };
        };
      }
    );
}
