{
  perSystem =
    {
      lib,
      pkgs,
      system,
      ...
    }:
    {
      # Blacksmith CLI (testbox: warm a remote CI box and run `vp test` /
      # `vp lint` / `vp build` against the local working tree). The vendor only
      # publishes a moving "latest" channel — there are no versioned URLs — so
      # these hashes must be refreshed when the CLI updates:
      #   for p in linux/amd64 linux/arm64 darwin/amd64 darwin/arm64; do
      #     curl -fsSL "https://clireleases.blacksmith.sh/cli/latest/$p/blacksmith.sha256"
      #   done
      packages.blacksmith =
        let
          selector = {
            x86_64-linux = {
              os = "linux";
              arch = "amd64";
              sha256 = "7f60f3b9f8d4d7644d9743f5d962acb3b3dbf675f51676702e5f292e02060bca";
            };
            aarch64-linux = {
              os = "linux";
              arch = "arm64";
              sha256 = "04c8d261526e23c7791f05b8acec8f02b9d1fe67c35a1adcf28076627327270a";
            };
            x86_64-darwin = {
              os = "darwin";
              arch = "amd64";
              sha256 = "47281f402ff223f85e5165ea9018cd0281a727f19c69af8121ae4b09658ad313";
            };
            aarch64-darwin = {
              os = "darwin";
              arch = "arm64";
              sha256 = "607b0f4413e426574527446c7718ea32587d57b24a3ea0749e1ab4138a426584";
            };
          };
          target = selector.${system} or (throw "blacksmith CLI: unsupported system ${system}");
        in
        pkgs.stdenvNoCC.mkDerivation {
          pname = "blacksmith-cli";
          version = "latest";

          src = pkgs.fetchurl {
            url = "https://clireleases.blacksmith.sh/cli/latest/${target.os}/${target.arch}/blacksmith";
            sha256 = target.sha256;
          };

          dontUnpack = true;

          nativeBuildInputs = [
            pkgs.makeWrapper
          ]
          ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.autoPatchelfHook ];

          installPhase = ''
            runHook preInstall
            install -Dm755 "$src" "$out/bin/blacksmith"
            runHook postInstall
          '';

          # The CLI shells out to rsync (testbox file sync, required) and to gh
          # (status reporting, optional) — bundle both so it works regardless of
          # the caller's PATH.
          postFixup = ''
            wrapProgram "$out/bin/blacksmith" \
              --prefix PATH : ${
                lib.makeBinPath [
                  pkgs.rsync
                  pkgs.gh
                ]
              }
          '';

          meta = {
            description = "Blacksmith CLI (testbox + CI tooling)";
            homepage = "https://docs.blacksmith.sh/blacksmith-testbox/overview";
            platforms = builtins.attrNames selector;
          };
        };
    };
}
