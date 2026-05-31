# Contributing

Thanks for helping improve Ox Content. This repository is a mixed Rust and TypeScript monorepo with Rust crates under `crates/`, npm packages under `npm/`, documentation under `docs/`, and runnable examples under `examples/`.

## Development Setup

Use the pinned development environment when possible:

```bash
nix develop
vp install
```

`nix develop` provides the Rust toolchain, Node.js, pnpm, Vite+, and build tools declared in `flake.nix`. `vp install` installs JavaScript dependencies through the workspace Vite+ wrapper. The canonical task graph lives in `vite.config.ts`.

## Common Commands

Run commands from the repository root:

```bash
vp fmt        # Format Rust and JS/TS sources
vp check      # Check Rust and JS/TS sources
vp dev        # Start the docs and playground dev servers
vp build      # Build Rust, npm packages, docs, and playground
```

Useful task-graph commands include:

```bash
vp run test
vp run test:rust
vp run test:ts
vp run fmt:check
vp run lint
vp run ready
vp run doc:cargo
vp run dev:docs
vp run dev:playground
vp run deploy#docs
vp run bench:parse
```

`vp run deploy#docs` builds the docs site and deploys `docs/dist/docs` to Void.
It builds with `OX_CONTENT_DOCS_BASE=/` and
`OX_CONTENT_DOCS_SITE_URL=https://ox-content.void.app` by default so asset URLs
resolve from the Void domain root. It uses the `VOID_PROJECT` environment
variable when set, otherwise it targets the `ox-content` Void project. Pass
additional Void CLI flags after the task specifier, for example
`vp run deploy#docs --debug`.

For allocation-aware performance work, use the in-tree profiler. It installs
a counting global allocator and turns on per-span timing for the parser and
renderer crates:

```bash
cargo run --release -p ox_content_profile_cli -- pipeline --gfm \
    --iters 200 --warmup 20 docs/content/api/types.md
```

See [docs/content/profiling.md](./docs/content/profiling.md) for the full
workflow and how to read the report.

For narrower Rust work, the underlying commands are also available:

```bash
cargo test --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Branches, Commits, and Pull Requests

- Create focused branches for each change.
- Use conventional commit messages such as `feat: add parser option`, `fix: preserve heading ids`, or `docs: clarify Vite setup`.
- Keep pull requests scoped to one purpose and describe the user-visible behavior, implementation notes, and verification performed.
- Link related issues when there are any.
- Include screenshots or rendered output for UI, documentation, or visual regression changes when that helps review.
- Do not mix unrelated formatting or cleanup with feature and bug-fix changes.

## Tests and Verification

Choose the smallest verification that covers the change, then broaden it when the impact crosses package boundaries.

- Rust crates: run `vp run test:rust` or targeted `cargo test -p <crate>`.
- TypeScript packages and Vite plugins: run `vp run test:ts`, `vp run check:ts`, or the relevant package command.
- Cross-package changes: run `vp run ready` before opening a PR.
- Visual regression changes: run `vp run test:vrt`; use `vp run test:vrt:update` only when snapshot updates are intentional.
- VS Code extension changes: run `vp run test:vscode-unit` for the pure-node helpers, and `vp run test:vscode` for the integration suite (the latter needs an X server — `xvfb-run` in CI, a real display locally). See [npm/vscode-ox-content/README.md](./npm/vscode-ox-content/README.md#testing) for the details.
- Docs-only changes: run the relevant docs build or dev task when the change affects rendering, navigation, examples, or code snippets.

## Documentation and Examples

Update docs and examples in the same PR when behavior changes. Common places to check are:

- `README.md` for top-level usage and development notes.
- `docs/content/` for the documentation site.
- `examples/` for runnable framework and plugin examples.
- Package READMEs under `npm/` or crate docs when a package-specific API changes.

Keep examples runnable and prefer existing patterns over introducing a new framework, runner, or helper for a small change.

## Release Notes

Use conventional commits because the release script groups changelog entries by commit type. When a change should be visible in release notes, make the commit message clear from a user perspective and mention migration or compatibility details in the PR description.

The release task is defined in `vite.config.ts` as `vp run release` and delegates to `scripts/release.ts`. Do not run a release from a feature PR unless the maintainer explicitly asks for it.

## Security Issues

Do not open a public issue or pull request with exploit details. If a `SECURITY.md` file is present, follow the reporting instructions there. If it is not present, contact the maintainer privately before sharing sensitive details.
