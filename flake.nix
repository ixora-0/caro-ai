{
  description = "Shell for caro-ai";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [ cargo rustc ];
        };
      }
  );
}
