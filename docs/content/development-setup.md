# Development Setup

This page is for contributors and for anyone building Ox Content itself from source.

If you just want to use the plugin or APIs, go back to [Getting Started](./getting-started.md).

## Prerequisites

Before you begin, ensure you have the following installed:

| Requirement   | Version | Installation                                                                         |
| ------------- | ------- | ------------------------------------------------------------------------------------ |
| **Rust**      | 1.83+   | Provided by `nix develop` (pinned in `flake.nix`) or [rustup.rs](https://rustup.rs/) |
| **Node.js**   | 24+     | Provided by `nix develop` or managed via `.node-version`                             |
| **Vite+**     | Latest  | Available as `vp` inside the dev shell                                               |
| **wasm-pack** | Latest  | Provided by `nix develop`; needed when you run `vp run build:wasm`                   |

## Clone and Bootstrap

```bash
# Clone the repository
git clone https://github.com/ubugeeei/ox-content.git
cd ox-content

# Enter the pinned development shell
nix develop

# Install JS dependencies
vp install

# Build all crates and packages
vp run build

# Run tests to verify installation
vp run test
```

## Workspace Tasks

Enter the pinned shell with `nix develop`, then run workspace tasks via `vp run <task>`.
The canonical task graph lives in `vite.config.ts`.

```bash
# Setup
vp install

# Building
vp run build
vp run build:rust
vp run build:rust-release
vp run build:napi
vp run build:npm
vp run build:wasm

# Testing
vp run test
vp run test:rust
vp run test:rust-verbose
vp run test:ts
vp run watch

# Code quality
vp run fmt
vp run fmt:check
vp run clippy
vp run lint
vp run ready

# Documentation
vp run doc:cargo
vp run doc:cargo-open

# Docs and examples
vp run dev
vp run dev:docs
vp run dev:playground
vp run playground
vp run integ-vue
vp run integ-react
vp run integ-svelte
vp run ssg-vite

# Benchmarks
vp run bench
vp run bench:rust
vp run bench:parse
vp run bench:bundle
```

## Project Structure

```text
ox-content/
├── Cargo.toml              # Workspace configuration
├── flake.nix               # Nix dev shell (Node.js, workspace bootstrap, Rust, Vite+ wrapper)
├── .node-version           # Node.js version for CI / setup-node compatibility
├── vite.config.ts          # Vite+ workspace task graph
├── crates/                 # Rust crates
│   ├── ox_content_allocator/   # Arena allocator
│   ├── ox_content_ast/         # AST node definitions
│   ├── ox_content_parser/      # Markdown parser
│   ├── ox_content_renderer/    # HTML renderer
│   ├── ox_content_search/      # Full-text search engine
│   ├── ox_content_napi/        # Node.js N-API bindings
│   ├── ox_content_wasm/        # WebAssembly bindings
│   ├── ox_content_og_image/    # OG image generation
│   └── ox_content_lsp/         # Unified language server
├── npm/                    # npm packages
│   ├── vite-plugin-ox-content/       # @ox-content/vite-plugin
│   ├── vite-plugin-ox-content-vue/   # @ox-content/vite-plugin-vue
│   ├── vite-plugin-ox-content-react/ # @ox-content/vite-plugin-react
│   ├── vite-plugin-ox-content-svelte/# @ox-content/vite-plugin-svelte
│   ├── unplugin-ox-content/          # @ox-content/unplugin
│   └── vscode-ox-content/            # VS Code extension
├── editors/                # Editor integrations
│   ├── zed/                # Zed extension
│   └── neovim/             # Neovim plugin
├── examples/               # Usage examples
├── docs/                   # Documentation site
└── .github/workflows/      # CI/CD
```

## Running Tests

### All Tests

```bash
vp run test

# or
cargo test --workspace
```

### Specific Crates

```bash
cargo test -p ox_content_parser
cargo test -p ox_content_renderer
```

### With Output

```bash
cargo test --workspace -- --nocapture
```

## Running the Docs and Playground

```bash
# Start docs and playground together
vp run dev

# Only the docs site
vp run dev:docs

# Only the playground
vp run playground
```

Then open [http://127.0.0.1:4173](http://127.0.0.1:4173) for the docs site and [http://127.0.0.1:5173](http://127.0.0.1:5173) for the playground.

## Running Benchmarks

```bash
vp run bench
vp run bench:rust
vp run bench:parse
vp run bench:bundle
```

The latest published benchmark snapshot lives on the home page under [Benchmarks](./index.md#benchmarks).

## Troubleshooting

### `cargo: command not found`

Ensure Rust is installed and in your `PATH`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### `nix: command not found`

Install Nix with the official installer, restart your shell, then re-enter the repo:

```bash
nix develop
```

### Build fails with linking errors

On Linux, you may need build essentials:

```bash
# Ubuntu / Debian
sudo apt-get install build-essential

# Fedora
sudo dnf groupinstall "Development Tools"
```

On macOS, install Xcode Command Line Tools:

```bash
xcode-select --install
```

### N-API build fails

Ensure you are on the expected Node.js version:

```bash
nix develop
node -v
vp run build:napi
```

If you manage Node.js outside Nix, match the version in `.node-version`.

### `wasm-pack: command not found`

The WASM build task expects `wasm-pack` to be available:

```bash
nix develop
vp run build:wasm
```

If you are not using Nix, install `wasm-pack` manually and make sure `wasm32-unknown-unknown` is available through `rustup`.

## Getting Help

- [GitHub Issues](https://github.com/ubugeeei/ox-content/issues)
- [Discussions](https://github.com/ubugeeei/ox-content/discussions)
