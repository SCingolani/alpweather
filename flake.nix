{
  description = "Weather along a GPX cycling route in the Alps and Dolomites";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "alpine-weather-route";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
          postInstall = pkgs.lib.optionalString (builtins.pathExists ./static) ''
            mkdir -p $out/share/alpine-weather-route
            cp -r static $out/share/alpine-weather-route/
          '';
          meta.mainProgram = "alpine-weather-route";
        };

        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.rustc
            pkgs.cargo
            pkgs.pkg-config
            pkgs.openssl
          ];
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };
      }) // {
        nixosModules.default = { pkgs, lib, ... }: {
          imports = [ ./nix/module.nix ];
          services.alpine-weather-route.package = lib.mkDefault self.packages.${pkgs.system}.default;
        };
        nixosModules.alpine-weather-route = self.nixosModules.default;
      };
}
