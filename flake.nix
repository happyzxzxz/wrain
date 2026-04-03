{
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
          pname = "wrain";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ 
            pkgs.pkg-config 
            pkgs.makeWrapper 
          ];

          buildInputs = [ 
            pkgs.wayland 
            pkgs.libxkbcommon 
            pkgs.libGL 
            pkgs.vulkan-loader 
            pkgs.alsa-lib 
          ];

          postInstall = ''
            mkdir -p $out/bin/assets
            cp -r assets/* $out/bin/assets/

            wrapProgram $out/bin/wrain \
              --set WRAIN_ASSET_PATH "$out/bin/assets" \
              --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath [ 
                pkgs.wayland pkgs.libxkbcommon pkgs.libGL pkgs.vulkan-loader pkgs.alsa-lib 
              ]}
          '';
        };
      });
}