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
      in
      {
        devShells.default = pkgs.mkShell {
          packages =
            [
              nodejs
              pnpm
              workspaceVp
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
          '';
        };

        formatter = pkgs.nixfmt;
      }
    );
}
