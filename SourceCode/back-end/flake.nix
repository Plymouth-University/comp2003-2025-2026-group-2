{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
          config = {
            allowUnfree = true;
          };
        };
        pkgsCrossAarch64 = import nixpkgs {
          inherit system;
          crossSystem = {
            config = "aarch64-unknown-linux-gnu";
            rust.rustcTarget = "aarch64-unknown-linux-gnu";
          };
          overlays = [ (import rust-overlay) ];
        };

        rustNative = (
          pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-std"
            ];
            targets = [ "aarch64-unknown-linux-gnu" ];
          }
        );
        craneLibNative = (crane.mkLib pkgs).overrideToolchain rustNative;
        swaggerUiZip = pkgs.fetchurl {
          url = "https://github.com/swagger-api/swagger-ui/archive/refs/tags/v5.17.14.zip";
          hash = "sha256-SBJE0IEgl7Efuu73n3HZQrFxYX+cn5UU5jrL4T5xzNw=";
        };
        commonArgs = {
          src = craneLibNative.cleanCargoSource ./.;
          strictDeps = true;
          doCheck = false;
          buildInputs = with pkgs; [ openssl ];
          nativeBuildInputs = with pkgs; [
            pkg-config
            openssl
            mold
            clang
            patchelf
          ];
          preBuild = ''
            mkdir -p /tmp/swagger-ui
            cp ${swaggerUiZip} /tmp/swagger-ui/v5.17.14.zip
            chmod 644 /tmp/swagger-ui/v5.17.14.zip
          '';
          SWAGGER_UI_DOWNLOAD_URL = "file:///tmp/swagger-ui/v5.17.14.zip";
          RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=mold";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [ pkgs.openssl ]}";
        };
        cargoArtifacts = craneLibNative.buildDepsOnly commonArgs;
        logSmartBackendNative = craneLibNative.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            doCheck = false;
            postInstall = ''
              patchelf --set-rpath "${pkgs.lib.makeLibraryPath [ pkgs.openssl ]}" $out/bin/logsmart-srv
            '';
          }
        );

        craneLibCrossAarch64 = (crane.mkLib pkgsCrossAarch64).overrideToolchain (p: p.rust-bin.stable.latest.default);
        crateExpressionCrossAarch64 =
          {
            openssl,
            pkg-config,
            stdenv,
          }:
          let
            cargoArtifactsCross = craneLibCrossAarch64.buildDepsOnly {
              src = craneLibCrossAarch64.cleanCargoSource ./.;
              strictDeps = true;
              doCheck = false;
              buildInputs = [ openssl ];
              nativeBuildInputs = [
                pkg-config
                openssl
              ];
              preBuild = ''
                mkdir -p /tmp/swagger-ui
                cp ${swaggerUiZip} /tmp/swagger-ui/v5.17.14.zip
                chmod 644 /tmp/swagger-ui/v5.17.14.zip
              '';
              SWAGGER_UI_DOWNLOAD_URL = "file:///tmp/swagger-ui/v5.17.14.zip";
              HOST_CC = "${pkgs.stdenv.cc}/bin/cc";
              OPENSSL_DIR = "${pkgs.openssl.out}";
              OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
              OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
              PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
              AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_DIR = "${openssl.out}";
              AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR = "${openssl.out}/lib";
              AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR = "${openssl.dev}/include";
              PKG_CONFIG_PATH_aarch64_unknown_linux_gnu = "${openssl.dev}/lib/pkgconfig";
              PKG_CONFIG_SYSROOT_DIR = "${stdenv.cc.libc}";
            };
          in
          craneLibCrossAarch64.buildPackage {
            inherit cargoArtifactsCross;
            src = craneLibCrossAarch64.cleanCargoSource ./.;
            strictDeps = true;
            doCheck = false;
            nativeBuildInputs = [
              pkg-config
              openssl
            ];
            buildInputs = [
              openssl
            ];
            preBuild = ''
              mkdir -p /tmp/swagger-ui
              cp ${swaggerUiZip} /tmp/swagger-ui/v5.17.14.zip
              chmod 644 /tmp/swagger-ui/v5.17.14.zip
            '';
            SWAGGER_UI_DOWNLOAD_URL = "file:///tmp/swagger-ui/v5.17.14.zip";
            HOST_CC = "${pkgs.stdenv.cc}/bin/cc";
            OPENSSL_DIR = "${pkgs.openssl.out}";
            OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
            OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_DIR = "${openssl.out}";
            AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR = "${openssl.out}/lib";
            AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR = "${openssl.dev}/include";
            PKG_CONFIG_PATH_aarch64_unknown_linux_gnu = "${openssl.dev}/lib/pkgconfig";
            PKG_CONFIG_SYSROOT_DIR = "${stdenv.cc.libc}";
          };
        logSmartBackendCrossAarch64 = pkgsCrossAarch64.callPackage crateExpressionCrossAarch64 { };

        deployAarch64Alias = pkgs.writeShellScriptBin "deploy-aarch64" ''
          nix build .#docker-image-aarch64 && docker load < result && docker push nullstring1/logsmart-srv:latest-aarch64
        '';

        dockerImageAarch64 = pkgs.dockerTools.buildLayeredImage {
          name = "nullstring1/logsmart-srv";
          tag = "latest-aarch64";
          contents = [
            logSmartBackendCrossAarch64
            pkgsCrossAarch64.openssl
          ];
          
          architecture = "arm64"; 
          
          config = {
            Cmd = [ "${logSmartBackendCrossAarch64}/bin/logsmart-srv" ];
            ExposedPorts = {
              "6767/tcp" = {};
            };
          };
        };

        dockerImagex86_64 = pkgs.dockerTools.buildLayeredImage {
          name = "nullstring1/logsmart-srv";
          tag = "latest-x86-64";
          contents = [
            logSmartBackendNative
            pkgs.openssl
          ];
          
          architecture = "amd64"; 
          
          config = {
            Cmd = [ "${logSmartBackendNative}/bin/logsmart-srv" ];
            ExposedPorts = {
              "6767/tcp" = {};
            };
          };
        };

        packages = {
          aarch64-linux = logSmartBackendCrossAarch64;
          x86_64-linux = logSmartBackendNative;
          aarch64-darwin = logSmartBackendNative;
          default = logSmartBackendNative;
          docker-image-aarch64 = dockerImageAarch64;
          docker-image-x86_64 = dockerImagex86_64;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustNative
            rust-analyzer
            openssl.dev
            pkgsCrossAarch64.openssl.dev
            libz
            pkg-config
            sqlx-cli
            mongodb-compass
            mold
            clang
            cachix
            deployAarch64Alias
          ];
          env = {
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            PKG_CONFIG_PATH_aarch64_unknown_linux_gnu = "${pkgsCrossAarch64.openssl.dev}/lib/pkgconfig";
            PKG_CONFIG_ALLOW_CROSS = "1";
            PKG_CONFIG_SYSROOT_DIR = "${pkgsCrossAarch64.stdenv.cc.libc}";
            AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR = "${pkgsCrossAarch64.openssl.dev}/include";
            AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR = "${pkgsCrossAarch64.openssl.out}/lib";
            AARCH64_UNKNOWN_LINUX_GNU_OPENSSL_DIR = "${pkgsCrossAarch64.openssl.out}";
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${pkgsCrossAarch64.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc";
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C linker=${pkgsCrossAarch64.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc -L ${pkgsCrossAarch64.xz.out}/lib -L ${pkgsCrossAarch64.openssl.out}/lib -C link-args=-Wl,--enable-new-dtags,-rpath,$ORIGIN/../lib:$ORIGIN/lib:/usr/lib/aarch64-linux-gnu:/lib/aarch64-linux-gnu:/usr/lib:/lib,--dynamic-linker=/lib/ld-linux-aarch64.so.1";
            CC_aarch64_unknown_linux_gnu = "${pkgsCrossAarch64.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc";
            CXX_aarch64_unknown_linux_gnu = "${pkgsCrossAarch64.stdenv.cc}/bin/aarch64-unknown-linux-gnu-g++";
            AR_aarch64_unknown_linux_gnu = "${pkgsCrossAarch64.stdenv.cc}/bin/aarch64-unknown-linux-gnu-ar";
          };
        };
        packages = packages;
        apps.default = {
          type = "app";
          program = "${logSmartBackendNative}/bin/logsmart-srv";
        };
        apps.aarch64-linux = {
          type = "app";
          program = "${logSmartBackendCrossAarch64}/bin/logsmart-srv";
        };
      }
    );
}
