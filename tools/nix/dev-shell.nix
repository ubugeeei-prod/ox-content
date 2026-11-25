{
  perSystem =
    {
      config,
      lib,
      pkgs,
      rustToolchain,
      ...
    }:
    let
      nodejs = pkgs.nodejs_26;
      pnpm = pkgs.pnpm;
    in
    {
      devShells.default = pkgs.mkShell {
        packages = [
          nodejs
          pnpm
          config.packages.vp
          config.packages.blacksmith
          rustToolchain
          config.packages.kache
          pkgs.wasm-pack
          pkgs.wasm-bindgen-cli
          pkgs.binaryen
          pkgs.cargo-watch
          pkgs.cargo-llvm-cov
          pkgs.git
          pkgs.gh
          pkgs.jq
          pkgs.pkg-config
          pkgs.rsync
        ]
        ++ lib.optionals pkgs.stdenv.hostPlatform.isDarwin [ pkgs.libiconv ];

        RUST_BACKTRACE = "1";
        RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

        shellHook = ''
          export OX_CONTENT_WORKSPACE_ROOT="$PWD"
          export PATH="$OX_CONTENT_WORKSPACE_ROOT/node_modules/.bin:$PATH"
          export PLAYWRIGHT_BROWSERS_PATH="$OX_CONTENT_WORKSPACE_ROOT/.cache/ms-playwright"
          export PUPPETEER_CACHE_DIR="$OX_CONTENT_WORKSPACE_ROOT/.cache/puppeteer"

          # Rust build accelerators (CONTRIBUTING.md, "Rust build accelerators"):
          # workspace crates compile with rustc's parallel frontend, and kache
          # shares compiled crates across worktrees and target directories.
          # Enter the shell with OX_CONTENT_RUSTC_FRONTEND_JOBS=1 or
          # OX_CONTENT_KACHE= (empty) to turn either off.
          case "''${RUSTC_WRAPPER:-}" in
            "" | */tools/scripts/rustc-wrapper.sh)
              export RUSTC_WRAPPER="$OX_CONTENT_WORKSPACE_ROOT/tools/scripts/rustc-wrapper.sh"
              ;;
          esac
          export OX_CONTENT_KACHE="''${OX_CONTENT_KACHE-${lib.getExe config.packages.kache}}"
          export OX_CONTENT_RUSTC_FRONTEND_JOBS="''${OX_CONTENT_RUSTC_FRONTEND_JOBS:-8}"

          echo "Ox Content dev shell ready."
          echo "rustc: parallel frontend x$OX_CONTENT_RUSTC_FRONTEND_JOBS for workspace crates''${OX_CONTENT_KACHE:+, cached by kache}"
          echo "Run: vp install"
          echo "Then: vp run ready"
          echo "Testbox: blacksmith auth login && vp run testbox:warmup"
        '';
      };
    };
}
