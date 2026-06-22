# Publishing Editor Extensions

This package is released through `.github/workflows/publish-editors.yml`.

## One-time setup

1. Create the VS Code Marketplace publisher `ubugeeei`.
2. Add repository secret `VSCE_PAT` with Marketplace Manage permission.
   `vsce` reads this token from the `VSCE_PAT` environment variable in CI.
3. Create the Open VSX namespace `ubugeeei`.
4. Add repository secret `OVSX_PAT`.
5. Optional for Zed: add `ZED_EXTENSIONS_PAT` with permission to push a branch
   to `zed-industries/extensions` and open a pull request. The Zed extension
   path includes its own `LICENSE` because Zed validates the license at the
   configured `path`.

References:

- VS Code Marketplace publishing:
  https://code.visualstudio.com/api/working-with-extensions/publishing-extension
- VS Code automated publishing:
  https://code.visualstudio.com/api/working-with-extensions/continuous-integration
- Open VSX publishing:
  https://github.com/eclipse-openvsx/openvsx/wiki/Publishing-Extensions
- Zed extension publishing:
  https://zed.dev/docs/extensions/developing-extensions#publishing-your-extension

## Release flow

1. Run the normal repository release:

   ```bash
   vp run release patch
   ```

   The release script updates `npm/vscode-ox-content/package.json`,
   `editors/zed/extension.toml`, and `editors/zed/Cargo.toml`.

2. The pushed `v*` tag starts `Publish editor extensions`.

3. The workflow builds platform-specific VSIX files with bundled
   `ox-content-lsp` binaries for:
   - `linux-x64`
   - `darwin-x64`
   - `darwin-arm64`
   - `win32-x64`

4. If `VSCE_PAT` is configured, the workflow publishes every VSIX to the VS
   Code Marketplace.

5. If `OVSX_PAT` is configured, the workflow publishes the same VSIX files to
   Open VSX.

6. The Zed job builds the extension for `wasm32-unknown-unknown` and uploads a
   source artifact. If `ZED_EXTENSIONS_PAT` is configured, it also opens a PR
   against `zed-industries/extensions` with:

   ```toml
   [ox-content]
   submodule = "extensions/ox-content"
   path = "editors/zed"
   version = "<version>"
   ```

## Manual dry run

Use the `Publish editor extensions` workflow dispatch without `publish` checked.
It packages VSIX and Zed artifacts but skips Marketplace/Open VSX publishing.

To publish from a manual dispatch, provide `version` and check `publish`.

## Local package smoke test

```bash
vp run build:lsp
vp run vscode:build
mkdir -p npm/vscode-ox-content/bin
cp target/release/ox-content-lsp npm/vscode-ox-content/bin/
(cd npm/vscode-ox-content && vp exec -- pnpm dlx @vscode/vsce package --no-dependencies)
```
