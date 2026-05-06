{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    back-end.url = "path:./back-end";
    front-end.url = "path:./front-end";
  };
  outputs =
    {
      self,
      nixpkgs,
      back-end,
      front-end,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          inputsFrom = [
            back-end.devShells.${system}.default
            front-end.devShells.${system}.default
          ];

        };
      }
    );
}
