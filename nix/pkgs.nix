{ inputs, ... }:
let
  root = ./..;
in
{
  perSystem =
    { system, ... }:
    let
      pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ inputs.rust-overlay.overlays.default ];
      };
    in
    {
      # Every module that needs a package set needs the same overlaid one, and
      # every module that needs a compiler needs the one rust-toolchain.toml
      # names. Both are resolved once here and handed out as module arguments.
      _module.args = {
        inherit pkgs;
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile (root + /rust-toolchain.toml);
      };
    };
}
