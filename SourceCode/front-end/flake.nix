{
  description = "SvelteKit development flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        chrome-playwright-browsers = pkgs.playwright-driver.browsers.override {
          withFirefox = false;
          withWebkit = false;
          withChromium = true;
          withChromiumHeadlessShell = true;
        };
        wayland-playwright-browsers =
          pkgs.runCommand "wayland-playwright-browsers"
            {
              nativeBuildInputs = [
                pkgs.makeWrapper
                pkgs.xorg.lndir
              ];
            }
            ''
              mkdir -p $out

              # 1. Loop through chromium and headless
              for browser_symlink in ${chrome-playwright-browsers}/*; do
                [ -e "$browser_symlink" ] || continue

                DIR_NAME=$(basename "$browser_symlink")
                REAL_DIR=$(readlink -f "$browser_symlink")

                mkdir -p "$out/$DIR_NAME"
                lndir -silent "$REAL_DIR" "$out/$DIR_NAME"
              done

              # 2. Find and wrap the chromium for wayland
              CHROME_BIN=$(find $out -type l -path "*/chromium-*/chrome-linux*/chrome" | head -n 1)
              if [ -n "$CHROME_BIN" ]; then
                REAL_CHROME=$(readlink -f "$CHROME_BIN")
                rm "$CHROME_BIN"
                makeWrapper "$REAL_CHROME" "$CHROME_BIN" \
                  --prefix LD_LIBRARY_PATH : "${
                    pkgs.lib.makeLibraryPath [
                      pkgs.wayland
                      pkgs.libGL
                      pkgs.libxkbcommon
                    ]
                  }:/run/opengl-driver/lib" \
                  --add-flags "--ozone-platform=wayland --enable-features=UseOzonePlatform,WaylandWindowDecorations"
              fi
            '';
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.bun
            pkgs.nodejs
          ];
          env = {
            PLAYWRIGHT_BROWSERS_PATH = "${wayland-playwright-browsers}";
            PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = "true";
          };
        };
      }
    );
}
