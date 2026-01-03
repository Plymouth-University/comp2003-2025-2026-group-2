{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, naersk, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
        config = {
          allowUnfree = true;
        };
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
      naersk-lib = naersk.lib.${system};
      naersk-lib-cross = (naersk.lib.${system}).override {
        cargo = pkgsCross.cargo;
        rustc = pkgsCross.rustc;
      };
      swaggerUiZip = pkgs.fetchurl {
        url = "https://github.com/swagger-api/swagger-ui/archive/refs/tags/v5.17.14.zip";
        hash = "sha256-SBJE0IEgl7Efuu73n3HZQrFxYX+cn5UU5jrL4T5xzNw=";
      };
      logSmartBackendAarch64 = naersk-lib-cross.buildPackage {
        src = ./.;
        buildInputs = with pkgsCross; [ openssl pkg-config ];
        nativeBuildInputs = with pkgsCross; [ pkg-config openssl cargo rustc patchelf ];
        runtimeDependencies = with pkgsCross; [ openssl ];
        SWAGGER_UI_DOWNLOAD_URL = "file://" + (pkgs.fetchurl {
          url = "https://github.com/swagger-api/swagger-ui/archive/refs/tags/v5.17.14.zip";
          hash = "sha256-SBJE0IEgl7Efuu73n3HZQrFxYX+cn5UU5jrL4T5xzNw=";
        });
        OPENSSL_DIR = "${pkgsCross.openssl.out}";
        OPENSSL_LIB_DIR = "${pkgsCross.openssl.out}/lib";
        OPENSSL_INCLUDE_DIR = "${pkgsCross.openssl.dev}/include";
        AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_DIR = "${pkgsCross.openssl.out}";
        AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR = "${pkgsCross.openssl.out}/lib";
        AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR = "${pkgsCross.openssl.dev}/include";
        PKG_CONFIG_PATH = "${pkgsCross.openssl.dev}/lib/pkgconfig";
        PKG_CONFIG_PATH_aarch64_unknown_linux_gnu = "${pkgsCross.openssl.dev}/lib/pkgconfig";
        PKG_CONFIG_SYSROOT_DIR = "${pkgsCross.stdenv.cc.libc}";
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc";
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C linker=${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc -L ${pkgsCross.openssl.out}/lib -C link-args=-Wl,--enable-new-dtags,-rpath,$ORIGIN/../lib:$ORIGIN/lib:/usr/lib/aarch64-linux-gnu:/lib/aarch64-linux-gnu:/usr/lib:/lib,--dynamic-linker=/lib/ld-linux-aarch64.so.1";
        CC_aarch64_unknown_linux_gnu = "${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc";
        CXX_aarch64_unknown_linux_gnu = "${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-g++";
        AR_aarch64_unknown_linux_gnu = "${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-ar";
        postInstall = ''
          patchelf --set-rpath "${pkgsCross.lib.makeLibraryPath [ pkgsCross.openssl ]}" $out/bin/logsmart-srv
        '';
      };
      logSmartBackend = naersk-lib.buildPackage {
        src = ./.;
        buildInputs = with pkgs; [ openssl ];
        nativeBuildInputs = with pkgs; [ pkg-config openssl mold clang patchelf ];
        runtimeDependencies = with pkgs; [ openssl ];
        preBuild = ''
          mkdir -p /tmp/swagger-ui
          cp ${swaggerUiZip} /tmp/swagger-ui/v5.17.14.zip
          chmod +w /tmp/swagger-ui/v5.17.14.zip
        '';
        SWAGGER_UI_DOWNLOAD_URL = "file:///tmp/swagger-ui/v5.17.14.zip";
        RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=mold";
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [ pkgs.openssl ]}";
        postInstall = ''
          patchelf --set-rpath "${pkgs.lib.makeLibraryPath [ pkgs.openssl ]}" $out/bin/logsmart-srv
        '';
      };
      packages = {
        aarch64-linux = logSmartBackendAarch64;
        x86_64-linux = logSmartBackend;
        default = logSmartBackend;
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
          mongodb-compass
          mold
          clang
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
      apps.default = {
        type = "app";
        program = "${logSmartBackend}/bin/logsmart-srv";
      };
      apps.aarch64-linux = {
        type = "app";
        program = "${logSmartBackendAarch64}/bin/logsmart-srv";
      };
    }
  );
}
