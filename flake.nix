{
  description = "Copetch - A custom system fetch written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
      pname = "copetch";
      version = "0.1.0";

      src = ./.;
      cargoLock = {
        lockFile = ./Cargo.lock;
      };
    };

    apps.${system}.default = {
      type = "app";
      program = "${self.packages.${system}.default}/bin/copetch";
    };
  };
}