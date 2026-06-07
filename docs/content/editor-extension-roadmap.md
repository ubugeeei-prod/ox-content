# Editor Extension Roadmap

This document tracks the production-readiness plan for the Ox Content editor
extensions. The primary targets are **VS Code** and **Neovim**; Zed is kept in
lock-step where possible but is not a release gate. Every feature must be
runnable from a CLI so it can be wired into CI without an editor.

The roadmap is intentionally split into small, conventional pull requests so
each one can land independently with its own tests and changelog entry. A PR
must not depend on a later one in the list.

## Architecture Principles

1. **One server, many clients.** All language intelligence lives in
   `ox_content_lsp` (Rust). Editor plugins are thin clients that only
   translate LSP capabilities into editor-native UI.
2. **Every feature ships a CLI counterpart.** If a check or generator only
   exists inside the LSP, it cannot run in CI. The minimum bar is one binary
   per feature (`ox-content-link-check`, `ox-content-textlint`, …) returning
   non-zero on failure with a stable text/JSON output.
3. **Native dependencies stay native.** Type-aware features that need
   TypeScript talk to `typescript-go` via the `corsa_client` Rust crate, not
   through Node. The LSP spawns a single `tsgo` subprocess per workspace and
   reuses it.
4. **Editor plugins stay testable headlessly.** VS Code uses
   `@vscode/test-cli` + `@vscode/test-electron`. Neovim uses
   `nvim --headless` + `busted`. Both run in CI on every PR.
5. **No editor-specific feature.** A feature ships in the LSP first, with a
   CLI, and only then surfaces in any editor. This keeps Neovim, Zed, and
   future editors at parity.

## Feature Matrix

| #   | Feature                                         | LSP                      | CLI                          | VS Code                  | Neovim                      | Status           |
| --- | ----------------------------------------------- | ------------------------ | ---------------------------- | ------------------------ | --------------------------- | ---------------- |
| 1   | Markdown preview (HMR)                          | push channel             | none                         | subscribed webview       | external browser, on-demand | needs CLI + nvim |
| 2   | i18n preview / completion                       | present                  | `ox-content-i18n`            | present                  | present                     | shipped          |
| 3   | MDC completion + type check                     | completion + diagnostics | `ox-content-mdc-check`       | completion + diagnostics | completion + diagnostics    | shipped          |
| 4   | Vue / React props completion + jump + typecheck | crate scaffold           | planned                      | planned                  | planned                     | scaffold landed  |
| 5   | Asset path completion + diagnostics             | completion provider      | via link checker             | completion + diagnostics | completion + diagnostics    | shipped          |
| 6   | Dead link checker                               | diagnostics              | `ox-content-link-check`      | diagnostics              | diagnostics                 | local: shipped   |
| 7   | textlint integration                            | on-save diagnostics      | via configured command       | enabled per setting      | enabled per setting         | shipped (opt-in) |
| 8   | Frontmatter schema completion + diagnostics     | present                  | none (validated through LSP) | present                  | present                     | needs CLI        |
| 9   | Document structure (outline + folding)          | symbols + folding ranges | none (unit-tested headless)  | outline + folding        | outline + folding           | shipped          |
| 10  | Document links (Markdown links + images)        | document link provider   | none (unit-tested headless)  | clickable links          | clickable links             | shipped          |
| 11  | Selection ranges (expand selection)             | selection range provider | none (unit-tested headless)  | expand/shrink selection  | expand/shrink selection     | shipped          |

## PR Sequence

Each item below is one PR. They are listed in execution order; later PRs may
depend on earlier ones, but never the reverse. Every PR ships:

- Rust unit tests for the new crate(s) and any new LSP request handlers.
- TypeScript Mocha tests for any VS Code wiring.
- Lua `busted` tests for any new Neovim surface.
- A CI job entry in `.github/workflows/ci.yml`.
- Documentation updates under `docs/content/` and the affected package README.

### 1. `test(vscode-ox-content): integration suite and CI job`

Foundation PR. Tightens the existing extension before adding features.

- Expand `npm/vscode-ox-content/src/test/extension.test.ts` into a suite that
  exercises: activation, command registration (LSP-advertised vs.
  webview-only), middleware guards, configuration round-trip, preview panel
  lifecycle, snippet contribution shape, and `oxContent.openPreview` happy and
  error paths.
- Add Node-only unit tests for `config.ts`, `client.ts` middleware predicates,
  `preview.ts` debouncing, and `constants.ts` (pure data) using `vitest`.
- New CI job `vscode-test` in `.github/workflows/ci.yml`, running on Linux
  with `xvfb-run` and pulling the LSP binary out of `target/release` to avoid
  `cargo run` at activation time.
- Document the test workflow in
  `npm/vscode-ox-content/README.md` and `CONTRIBUTING.md`.

### 2. `feat(lsp): preview HMR channel`

Replace the polling refresh path with an explicit push channel.

- ✅ LSP notification `oxContent/previewDidChange` with `{ uri, html, title }` payload
  (shipped, see `crates/ox_content_lsp/src/backend/commands.rs`).
- ✅ `oxContent.previewSubscribe` / `oxContent.previewUnsubscribe`
  execute-command handlers.
- ✅ VS Code webview subscribes on open, unsubscribes on dispose, and
  listens for `oxContent/previewDidChange` instead of debouncing on
  `onDidChangeTextDocument`.
- Pending follow-up: CLI `ox-content-preview` that hosts an SSE endpoint
  backed by the same renderer (useful for `--watch` workflows and for the
  Neovim browser preview). Tracked as a separate PR so this one stays
  focused on the LSP push channel.
- Pending follow-up: Neovim preview opens the SSE URL instead of writing
  a temp file when `auto_refresh = true`.

### 3. `feat(link-checker): new crate, LSP integration, CLI`

- ✅ New crate `ox_content_link_checker` with deterministic local link
  resolution (relative paths, self-anchors, image targets). Offline-only
  by design; ships with 11 unit tests covering every link form documented
  in the crate README.
- ✅ CLI `ox-content-link-check [paths…] [--src-dir DIR] [--ignore PATTERN]
[--format text|json]` with exit-code-1-on-error semantics for CI.
- ✅ LSP diagnostics under `source: "ox-content-link"`, wired into the
  per-document diagnostic publish path so they appear alongside parse,
  frontmatter, and MDC errors without an extra round trip.
- Pending follow-ups:
  - HTTP head-check pool behind a `http-check` feature flag.
  - Reference-link expansion (currently blocked on the parser).
  - Cross-file anchor resolution (currently emits a warning).
  - GitHub Actions snippet on the docs site.

### 4. `feat(lsp): asset path completion and diagnostics`

- ✅ LSP completion provider triggered on image and link openers
  (`![…](`, `[…](`, `<img src="`, `<a href="`, also `<video|audio|source|iframe|picture src="`).
  Image-context triggers narrow the file list to known media
  extensions; link-context triggers list every file plus directories.
- ✅ Hidden-file filtering, prefix matching, directory entries
  rendered as `name/`. UTF-16 cursor handling for international
  characters (covered by `line_prefix` tests).
- ✅ Diagnostics for missing assets are already produced by the
  link checker (see PR #3 above) since image targets flow through
  the same resolver. No new CLI surface needed.
- Pending follow-up: per-feature `oxContent.asset.srcDir` setting
  separate from the workspace root, for repos that keep static
  assets under a `public/` subdirectory.

### 5. `feat(mdc): component name and attribute completion`

- ✅ New `ox_content_mdc_checker::Registry` (de)serializes a JSON
  index of components and their attributes. Deterministic iteration
  order via `BTreeMap`, lenient parsing (unknown fields tolerated).
- ✅ LSP completion: component names after `<Foo|`, attribute names
  inside `<Foo |…>`. Inside-quote and post-`=` positions are skipped
  to avoid noise. Attribute insertion uses snippet syntax
  (`name="$0"`) so the cursor lands inside the value.
- ✅ Registry path is configurable via the `mdcComponents`
  initialization option, `mdc.components` in the workspace config
  file, or `OX_CONTENT_MDC_COMPONENTS` env var (in that order).
- Pending follow-ups:
  - Hover documentation for an MDC tag the cursor sits on
    (registry already exposes the data).
  - Diagnostic for using an unknown component (opt-in only — would
    be noisy for projects with partial registries).
  - Framework-specific auto-discovery (Nuxt content, Astro, etc.)
    that builds the registry without a hand-written JSON file.

### 6. `feat(component-resolver): Vue and React props via corsa_client`

The single largest item on the roadmap. Lands behind a `tsgo` opt-in
setting so users without `typescript-go` available are not affected.
Split into three sequential PRs:

#### 6a. Scaffold (`feat(component-resolver): scaffold crate`)

- ✅ New crate `ox_content_component_resolver` registered in the
  workspace.
- ✅ Public types: `Resolver`, `ResolverConfig`, `ResolvedComponent`,
  `ResolvedProp`, `Location`, `Error`.
- ✅ `corsa_client = "0.10"` wired through and proven to build
  against the workspace.
- ✅ Scaffold returns `Error::NotImplemented` so editor integrations
  can develop against the public types before the implementation
  lands.
- ✅ 5 unit tests pin the scaffold contract (missing-tsgo,
  relative-path, NotImplemented, serde round-trip, config builder).

#### 6b. Resolver implementation (planned follow-up)

- Open `component_file` as a tsgo virtual document.
- Locate the default export and extract the props type (TS `Props`,
  `defineProps<…>()`, React.FC `<Props>`).
- Enumerate prop members → name, type string, optionality, JSDoc,
  declaration location.
- One `tsgo` process shared per workspace; lifecycle owned by the
  resolver, not the editor.
- Integration test gated on `OX_CONTENT_TSGO_PATH` env var.

#### 6c. LSP + CLI integration (planned follow-up)

- LSP wires completion, hover, go-to-definition, and diagnostics on
  top of the resolver for MDC/MDX files.
- New CLI `ox-content-component-check` for CI.
- VS Code: `oxContent.components.tsgoPath`,
  `oxContent.components.tsconfig`, `oxContent.components.enabled`.
- Document the resolution model in `docs/content/component-resolution.md`.

### 7. `feat(textlint): sidecar integration`

- ✅ New `ox_content_lsp::textlint` module spawns
  `<command> --format json --stdin --stdin-filename <path>` and
  parses the per-file message array into LSP diagnostics under
  `source: "textlint"`. Runs **on save only** so the typing path
  stays fast (textlint can take a few hundred ms per file).
- ✅ Opt-in via `oxContent.textlintEnabled` initialization option,
  `textlint.enabled` in the workspace config, or
  `OX_CONTENT_TEXTLINT_ENABLED=1`. Off by default — textlint is
  heavy and noisy for projects that don't use it.
- ✅ Custom command override via `oxContent.textlintCommand` /
  `textlint.command` / `OX_CONTENT_TEXTLINT_COMMAND`. Empty falls
  back to `npx textlint`. Shell-style quoting supported.
- ✅ VS Code: `oxContent.textlint.enabled` /
  `oxContent.textlint.command` settings forward into the
  initialization options.
- ✅ 11 unit tests cover JSON parsing, severity mapping,
  zero-indexed coordinates, the missing-rule-id and unknown-severity
  cases, shlex-style command splitting, and the disabled /
  missing-binary subprocess paths.
- Pending follow-ups: code actions for textlint `--fix` suggestions,
  dedicated `ox-content-textlint` CLI (currently the user runs
  textlint directly), debounce / cancellation between rapid saves.

### 8. `feat(nvim): polish, parity, and busted suite`

- Add user commands for every new feature shipped above
  (`:OxContentLinkCheck`, `:OxContentAssetCheck`, `:OxContentTextlint`,
  `:OxContentComponentCheck`).
- Move client setup into a single composable surface so users can disable
  individual features.
- Add `busted` tests under `editors/neovim/tests/` and a CI job
  `neovim-test` running `nvim --headless -c "PlenaryBustedDirectory ..."`.
- Refresh the README with the new command surface.

### 9. `chore(release): VS Code Marketplace and OpenVSX publish workflow`

Final shipping PR. Does not change runtime behavior.

- New workflow `.github/workflows/publish-vscode.yml` triggered by tags of
  the form `vscode-v*`. Downloads pre-built `ox-content-lsp` artifacts from
  the `publish.yml` job for `linux-x64`, `linux-arm64`, `darwin-x64`,
  `darwin-arm64`, and `win32-x64`. Bundles each into a platform-specific
  VSIX and publishes via `vsce publish --packagePath` and
  `ovsx publish --packagePath` with provenance.
- Document the operational setup in
  `npm/vscode-ox-content/PUBLISHING.md`: publisher creation,
  `VSCE_PAT` and `OVSX_PAT` secrets, icon and README requirements,
  versioning policy, and how to issue a manual hotfix.

## Out of Scope

- IDEA / WebStorm plugin. Not on the roadmap; we expose only the LSP surface
  and let third parties wrap it.
- Bundled `tsgo` binary. The component resolver requires the user to provide
  a `typescript-go` build; we will not vendor it.
- Hosted preview service. Preview HMR runs locally only.

## Tracking

Progress is tracked through the conventional commit log: each PR above maps
1:1 to a `feat:` / `test:` / `chore:` commit on `main`. This document is
updated in the same PR when an item lands.
