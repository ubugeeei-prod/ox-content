{
  perSystem =
    {
      lib,
      pkgs,
      system,
      ...
    }:
    let
      # kache: a content-addressed rustc cache shared by every worktree and
      # target directory on the machine. The dev shell routes rustc through it
      # (tools/scripts/rustc-wrapper.sh); CI does not, because it rewrites the
      # paths it hashes, so its outputs are not byte-identical to an uncached
      # build. Pinned to an exact release; the hashes are the release's own
      # `.sha256` files.
      kacheVersion = "2.3.1";
      kacheArtifacts = {
        aarch64-darwin = {
          triple = "aarch64-apple-darwin";
          hash = "sha256-viLTAsQCOLOnX5aVrRgCLlo64ygwDCdKE4bEa6DmuNE=";
        };
        aarch64-linux = {
          triple = "aarch64-unknown-linux-musl";
          hash = "sha256-/dOmWIRM5Xw5HSMHPSUOVKH8Fk8pFzbI0Yd1ZEQ4R18=";
        };
        x86_64-linux = {
          triple = "x86_64-unknown-linux-musl";
          hash = "sha256-bx/z+fr1qbsOOFAcW/6atbSpgINPHINdCa1bD9+bark=";
        };
      };
      artifact = kacheArtifacts.${system} or (throw "kache: unsupported system ${system}");
    in
    {
      packages.kache = pkgs.stdenvNoCC.mkDerivation {
        pname = "kache";
        version = kacheVersion;
        src = pkgs.fetchurl {
          url = "https://github.com/kunobi-ninja/kache/releases/download/v${kacheVersion}/kache-${artifact.triple}.tar.gz";
          inherit (artifact) hash;
        };
        sourceRoot = ".";
        # kache and the engine it launches must come from the same release.
        installPhase = ''
          runHook preInstall
          install -Dm755 kache $out/bin/kache
          install -Dm755 kache-engine $out/bin/kache-engine
          runHook postInstall
        '';
        meta = {
          description = "Compiler cache for Rust, C/C++ and CUDA";
          homepage = "https://github.com/kunobi-ninja/kache";
          license = lib.licenses.asl20;
          mainProgram = "kache";
          platforms = builtins.attrNames kacheArtifacts;
        };
      };
    };
}
