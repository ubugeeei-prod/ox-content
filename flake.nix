{
  description = "Ox Content development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";

    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    let
      overlays = [ rust-overlay.overlays.default ];
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        lib = pkgs.lib;
        nodejs = pkgs.nodejs_24;
        pnpm = pkgs.pnpm;
        rustToolchain = pkgs.rust-bin.stable."1.92.0".default.override {
          extensions = [
            "clippy"
            "rust-src"
            "rustfmt"
          ];
          targets = [ "wasm32-unknown-unknown" ];
        };
        workspaceVp = pkgs.writeShellApplication {
          name = "vp";
          runtimeInputs = [
            nodejs
            pnpm
          ];
          text = ''
            workspace_root="''${OX_CONTENT_WORKSPACE_ROOT:-$PWD}"

            if [ -x "$workspace_root/node_modules/.bin/vp" ]; then
              exec "$workspace_root/node_modules/.bin/vp" "$@"
            fi

            if [ "$#" -gt 0 ] && [ "$1" = "install" ]; then
              echo "Bootstrapping workspace dependencies with pnpm install --frozen-lockfile..." >&2
              exec pnpm --dir "$workspace_root" install --frozen-lockfile
            fi

            cat >&2 <<'EOF'
            Local vite-plus is not installed yet.

            Run this inside the Nix shell:
              vp install

            Or bootstrap manually:
              pnpm install --frozen-lockfile
            EOF
            exit 127
          '';
        };

        # Blacksmith CLI (testbox: warm a remote CI box and run `vp test` /
        # `vp lint` / `vp build` against the local working tree). The vendor only
        # publishes a moving "latest" channel — there are no versioned URLs — so
        # these hashes must be refreshed when the CLI updates:
        #   for p in linux/amd64 linux/arm64 darwin/amd64 darwin/arm64; do
        #     curl -fsSL "https://clireleases.blacksmith.sh/cli/latest/$p/blacksmith.sha256"
        #   done
        blacksmith =
          let
            selector = {
              x86_64-linux = {
                os = "linux";
                arch = "amd64";
                sha256 = "38f1c7aef1a36c87e40b9b50bc573ecdbbb785de46b2825e95cb9abafa359bdd";
              };
              aarch64-linux = {
                os = "linux";
                arch = "arm64";
                sha256 = "f8705a89bda30c40aed542319165f48cee47e96fbd5e54ff95e079528c3c872e";
              };
              x86_64-darwin = {
                os = "darwin";
                arch = "amd64";
                sha256 = "6210b98c2b8b7903479456277753dd0959ed34f0226e9d5c40021c97a246f4e1";
              };
              aarch64-darwin = {
                os = "darwin";
                arch = "arm64";
                sha256 = "a3367ab4151b12a4ea771515c314a0d48b62294c0b671325d42731f2be973f66";
              };
            };
            target =
              selector.${system}
                or (throw "blacksmith CLI: unsupported system ${system}");
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
            ] ++ lib.optionals pkgs.stdenv.isLinux [ pkgs.autoPatchelfHook ];

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
                --prefix PATH : ${lib.makeBinPath [ pkgs.rsync pkgs.gh ]}
            '';

            meta = {
              description = "Blacksmith CLI (testbox + CI tooling)";
              homepage = "https://docs.blacksmith.sh/blacksmith-testbox/overview";
              platforms = builtins.attrNames selector;
            };
          };
      in
      {
        devShells.default = pkgs.mkShell {
          packages =
            [
              nodejs
              pnpm
              workspaceVp
              blacksmith
              rustToolchain
              pkgs.rust-analyzer
              pkgs.wasm-pack
              pkgs.wasm-bindgen-cli
              pkgs.binaryen
              pkgs.cargo-watch
              pkgs.cargo-llvm-cov
              pkgs.git
              pkgs.jq
              pkgs.pkg-config
              pkgs.rsync
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];

          RUST_BACKTRACE = "1";
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            export OX_CONTENT_WORKSPACE_ROOT="$PWD"
            export PATH="$OX_CONTENT_WORKSPACE_ROOT/node_modules/.bin:$PATH"
            export PLAYWRIGHT_BROWSERS_PATH="$OX_CONTENT_WORKSPACE_ROOT/.cache/ms-playwright"
            export PUPPETEER_CACHE_DIR="$OX_CONTENT_WORKSPACE_ROOT/.cache/puppeteer"

            echo "Ox Content dev shell ready."
            echo "Run: vp install"
            echo "Then: vp run ready"
            echo "Testbox: blacksmith auth login && blacksmith testbox warmup --workflow testbox.yml"
          '';
        };

        formatter = pkgs.nixfmt;
      }
    );
}
