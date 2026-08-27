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

          echo "Ox Content dev shell ready."
          echo "Run: vp install"
          echo "Then: vp run ready"
          echo "Testbox: blacksmith auth login && vp run testbox:warmup"
        '';
      };
    };
}
