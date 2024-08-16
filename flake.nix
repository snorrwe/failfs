{
  description = "Rust devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell
          {
            buildInputs = [
              just
              (rust-bin.nightly.latest.default.override {
                extensions = [ "rust-src" "rust-analyzer" ];
                targets = [ ];
              })
              cargo-edit
              fuse3
              pkg-config
              tree
            ];
            LD_LIBRARY_PATH = lib.makeLibraryPath ([ fuse3 ]);
          };
      }
    );
}
