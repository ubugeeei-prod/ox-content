{ inputs, ... }:
let
  root = ./..;
in
{
  perSystem =
    {
      lib,
      pkgs,
      rustToolchain,
      ...
    }:
    let
      # crane builds with the toolchain rust-toolchain.toml names, not the one
      # nixpkgs happens to carry, so the package and the dev shell compile with
      # the same rustc.
      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

      ox-content = import (root + /default.nix) { inherit craneLib pkgs root; };
    in
    {
      packages = {
        inherit ox-content;
        default = ox-content;
      };

      apps = lib.genAttrs (lib.attrNames ox-content.passthru.binaries) (name: {
        type = "app";
        program = lib.getExe' ox-content name;
      });
    };
}
