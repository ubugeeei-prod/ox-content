import { defineConfig } from "@vscode/test-cli";

// Configuration for `pnpm --filter vscode-ox-content test`. Tests run inside
// a real VS Code Electron process so they exercise the actual extension
// activation path, registered commands, and snippet contributions.
//
// LSP-dependent assertions skip themselves when `ox-content-lsp` is not on
// the runner — the test workspace is configured with an absolute path that
// resolves to `target/debug/ox-content-lsp` when present.
export default defineConfig({
  files: "dist/test/**/*.test.js",
  // Use the source-tree fixtures directly so `tsc` doesn't have to copy
  // `.md` / `.json` files into `dist`. The workspace just needs to exist;
  // the tests open files via vscode.workspace.openTextDocument(...).
  workspaceFolder: "src/test/fixtures/workspace",
  mocha: {
    // The test file uses `suite`/`test`/`suiteSetup`, which are TDD-style
    // globals. Mocha defaults to BDD (`describe`/`it`/`before`).
    ui: "tdd",
    timeout: 30_000,
    color: true,
  },
});
