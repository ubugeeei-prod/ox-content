{
  perSystem =
    { pkgs, ... }:
    let
      nodejs = pkgs.nodejs_26;
      pnpm = pkgs.pnpm;
    in
    {
      packages.vp = pkgs.writeShellApplication {
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
    };
}
