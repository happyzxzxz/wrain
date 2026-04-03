{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        runtimeLibs = with pkgs; [
          libGL libxkbcommon wayland vulkan-loader alsa-lib
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ pkg-config cargo rustc ];
          buildInputs = runtimeLibs;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
        };
      });
}