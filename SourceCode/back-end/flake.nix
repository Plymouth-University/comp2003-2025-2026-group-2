{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
      rust = (pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "rust-std" ];
        targets = [ "aarch64-unknown-linux-gnu" ];
      });
    in
    {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [ 
          rust
          rust-analyzer
          openssl.dev
          pkg-config
        ];
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
      };
    }
  );
}
