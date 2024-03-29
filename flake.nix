{
  description = "tq";

  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, utils, naersk }: {
    overlays = rec {
      expects-naersk = final: prev: {
        tq = final.naersk.buildPackage {
          pname = "tq";
          root = ./.;
          buildInputs = [ final.makeWrapper ];
          postInstall = ''
            wrapProgram "$out/bin/tq" \
              --prefix PATH : ${final.jq}/bin
          '';
        };
      };

      default = final: prev: {
        inherit (prev.appendOverlays [
          naersk.overlay
          expects-naersk
        ]) tq;
      };
    };
  } // utils.lib.eachDefaultSystem (system: with import nixpkgs
    { overlays = [ self.overlays.default ]; inherit system; }; rec {
    packages.default = tq;

    devShells.default = mkShell {
      packages = [
        cargo
        cargo-watch
        (writeShellScriptBin "jq" ''
          echo "$@"
          ${jq}/bin/jq "$@"
        '')
        rust-analyzer
        rustc
        rustfmt
      ];
      shellHook = ''
        export RUST_SRC_PATH=${rustPlatform.rustLibSrc}
      '';
    };
  });
}
