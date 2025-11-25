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
      pkgsCross = import nixpkgs {
        inherit system;
        crossSystem = {
          config = "aarch64-unknown-linux-gnu";
          rust.rustcTarget = "aarch64-unknown-linux-gnu";
        };
      };
      rust = (pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "rust-std" ];
        targets = [ "aarch64-unknown-linux-gnu" ];
      });
      logSmartBackend = pkgsCross.rustPlatform.buildRustPackage {
        pname = "logsmart-backend";
        version = "latest";
        src = ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };
        installTargets = [ "logsmart-srv" ];
        nativeBuildInputs = [ pkgs.pkg-config pkgs.openssl ];
        buildInputs = [ pkgsCross.openssl ];
        SWAGGER_UI_DOWNLOAD_URL = "file://" + pkgs.fetchurl {
          url = "https://github.com/swagger-api/swagger-ui/archive/refs/tags/v5.17.14.zip";
          hash = "sha256-SBJE0IEgl7Efuu73n3HZQrFxYX+cn5UU5jrL4T5xzNw=";
        };
        HOST_CC = "${pkgs.stdenv.cc}/bin/cc";
        OPENSSL_DIR = "${pkgs.openssl.out}";
        OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
        AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_DIR = "${pkgsCross.openssl.out}";
        AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR = "${pkgsCross.openssl.out}/lib";
        AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR = "${pkgsCross.openssl.dev}/include";
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        PKG_CONFIG_PATH_aarch64_unknown_linux_gnu = "${pkgsCross.openssl.dev}/lib/pkgconfig";
        PKG_CONFIG_ALLOW_CROSS = "1";
        PKG_CONFIG_SYSROOT_DIR = "${pkgsCross.stdenv.cc.libc}";
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc";
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C linker=${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc -L ${pkgsCross.openssl.out}/lib -C link-args=-Wl,--enable-new-dtags,-rpath,$ORIGIN/../lib:$ORIGIN/lib:/usr/lib/aarch64-linux-gnu:/lib/aarch64-linux-gnu:/usr/lib:/lib,--dynamic-linker=/lib/ld-linux-aarch64.so.1";
      };
      packages = {
        aarch64-linux = logSmartBackend;
      };
    in
    {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [ 
          rust
          rust-analyzer
          openssl.dev
          pkgsCross.openssl.dev
          libz
          pkg-config
          sqlx-cli
        ];
        env = {
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          PKG_CONFIG_PATH_aarch64_unknown_linux_gnu = "${pkgsCross.openssl.dev}/lib/pkgconfig";
          PKG_CONFIG_ALLOW_CROSS = "1";
          PKG_CONFIG_SYSROOT_DIR = "${pkgsCross.stdenv.cc.libc}";
          AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR = "${pkgsCross.openssl.dev}/include";
          AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR = "${pkgsCross.openssl.out}/lib";
          AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_DIR = "${pkgsCross.openssl.out}";
          CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc";
          CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C linker=${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc -L ${pkgsCross.xz.out}/lib -L ${pkgsCross.openssl.out}/lib -C link-args=-Wl,--enable-new-dtags,-rpath,$ORIGIN/../lib:$ORIGIN/lib:/usr/lib/aarch64-linux-gnu:/lib/aarch64-linux-gnu:/usr/lib:/lib,--dynamic-linker=/lib/ld-linux-aarch64.so.1";
          CC_aarch64_unknown_linux_gnu = "${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc";
          CXX_aarch64_unknown_linux_gnu = "${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-g++";
          AR_aarch64_unknown_linux_gnu = "${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-ar";
        };
      };
      packages = packages;
    }
  );
}
