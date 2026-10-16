# Publishing Editor Extensions

This package is released through `.github/workflows/publish-editors.yml`.

## One-time setup

1. Create the VS Code Marketplace publisher `ubugeeei`.
2. Add repository secret `VSCE_PAT` with Marketplace Manage permission.
   `vsce` reads this token from the `VSCE_PAT` environment variable in CI.
3. Create the Open VSX namespace `ubugeeei`.
4. Sign the Open VSX Publisher Agreement and publish the first reviewed VSIX
   with a local PAT. Then register `publish-editors.yml`, repository
   `ubugeeei-prod/ox-content`, environment `open-vsx` as its trusted publisher.
   Subsequent CI releases use OIDC; no `OVSX_PAT` secret is needed.
5. Zed registration requires a human who understands the extension to submit
   and explain the PR under its [AI policy](https://github.com/zed-industries/extensions/blob/main/AI_POLICY.md).
   The workflow prepares a patch for review. The Zed extension includes its own
   `LICENSE` because Zed validates the configured `path`.

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

5. The workflow publishes the same VSIX files to Open VSX with trusted publishing.
   Initial registration must be complete before this can succeed.

6. The Zed job builds the extension for `wasm32-unknown-unknown` and uploads a
   source artifact. The `zed-registry-patch` artifact contains a registration
   patch against `zed-industries/extensions`, pinned to the release commit:

   ```toml
   [ox-content]
   submodule = "extensions/ox-content"
   path = "editors/zed"
   version = "<version>"
   ```

Download `zed-registry-patch`, clone your fork of the Zed registry, and check out
its `registry-base.txt` commit. Apply the patch with `git apply --index`, then run
`git submodule update --init extensions/ox-content`. Review the extension source,
commit the changes, and submit the PR with your own explanation. Write maintainer
responses yourself; the workflow does not submit or comment on your behalf.

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
