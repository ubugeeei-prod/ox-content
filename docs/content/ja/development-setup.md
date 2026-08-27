# 開発環境のセットアップ

このページは、コントリビューターと、Ox Content 自体をソースからビルドする人向けです。

プラグインや API を使うだけなら、[はじめる](./getting-started.md) に戻ってください。

## 前提条件

始める前に、次がインストールされていることを確認してください。

| 要件          | バージョン | インストール                                                                                       |
| ------------- | ---------- | -------------------------------------------------------------------------------------------------- |
| **Rust**      | 1.95+      | `nix develop` が提供します（`rust-toolchain.toml` で固定）。または [rustup.rs](https://rustup.rs/) |
| **Node.js**   | 26+        | `nix develop` が提供するか、`.node-version` で管理します                                           |
| **Vite+**     | 最新       | 開発シェル内では `vp` として使えます                                                               |
| **wasm-pack** | 最新       | `nix develop` が提供します。`vp run build:wasm` を回すときに必要です                               |

## クローンとブートストラップ

```bash
# Clone the repository
git clone https://github.com/ubugeeei-prod/ox-content.git
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

## ワークスペースタスク

`nix develop` で固定シェルに入り、`vp run <task>` でワークスペースタスクを実行します。
正規のタスクグラフは `vite.config.ts` にあります。

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
vp run check:panic-constructs
vp run ready

# Documentation
vp run doc:cargo
vp run doc:cargo-open
vp run deploy#docs

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

## プロジェクト構成

```text
ox-content/
├── Cargo.toml              # Workspace configuration
├── flake.nix               # Nix dev shell (Node.js, workspace bootstrap, Rust, Vite+ wrapper)
├── rust-toolchain.toml     # Rust channel, components, and targets for Nix and rustup alike
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
│   ├── vite-plugin-ox-content-solid/ # @ox-content/vite-plugin-solid
│   ├── unplugin-ox-content/          # @ox-content/unplugin
│   └── vscode-ox-content/            # VS Code extension
├── editors/                # Editor integrations
│   ├── zed/                # Zed extension
│   └── neovim/             # Neovim plugin
├── examples/               # Usage examples
├── docs/                   # Documentation site
└── .github/workflows/      # CI/CD
```

## テストを実行する

### すべてのテスト

```bash
vp run test

# or
cargo test --workspace
```

### 特定の crate

```bash
cargo test -p ox_content_parser
cargo test -p ox_content_renderer
```

### 出力付き

```bash
cargo test --workspace -- --nocapture
```

## ドキュメントとプレイグラウンドを動かす

```bash
# Start docs and playground together
vp run dev

# Only the docs site
vp run dev:docs

# Only the playground
vp run playground
```

ドキュメントサイトは [http://127.0.0.1:4173](http://127.0.0.1:4173)、プレイグラウンドは [http://127.0.0.1:5173](http://127.0.0.1:5173) を開いてください。

## ドキュメントを Void へデプロイする

ドキュメントサイトを Void へデプロイするには:

```bash
vp run deploy#docs
```

このタスクは `void deploy` の前に Rust ワークスペースとローカル npm パッケージをビルドし、`https://ox-content.void.app` でアセットが正しく解決されるようルートの base パスを使います。

環境変数と上書きは [ドキュメントのデプロイ](./deployment.md) を見てください。

## ベンチマークを実行する

```bash
vp run bench
vp run bench:rust
vp run bench:parse
vp run bench:bundle
```

フィクスチャのバンドル gzip、描画 HTML gzip、ビルド時間、初期リクエスト、実行時下限の絶対天井は `benchmarks/perf-budgets.json` にあります。JSON 掃引のあとに次で確認します。

```bash
node benchmarks/bundle-size/measure.mjs --json /tmp/bundle.json
node benchmarks/bundle-size/check-budgets.mjs --bundle /tmp/bundle.json
```

PR Benchmark ジョブは head の測定に対してこの検査を走らせます。意図した増大は同じ PR で天井を上げるか、`benchmark-regression-accepted` ラベルを付けます。[パフォーマンス](./performance.md) を見てください。

コミットするベンチマーク表とチャートには、手元のマシンではなく Blacksmith ベースの docs 更新ワークフローを使ってください。

```bash
gh workflow run benchmark-docs.yml --ref main -f runs=7
```

このワークフローは `blacksmith-32vcpu-ubuntu-2404` で動き、更新した `README.md`、`docs/content/performance.md`、ベンチマーク SVG を含む PR を開きます。同じ生成器をローカルで回すには:

```bash
vp run bench:docs
```

Blacksmith の testbox ランナーに対する素早いリモート確認には:

```bash
blacksmith auth login
TESTBOX_ID=$(blacksmith testbox warmup .github/workflows/testbox.yml --job testbox --idle-timeout 60)
blacksmith testbox run --id "$TESTBOX_ID" "vp run bench"
blacksmith testbox stop --id "$TESTBOX_ID"
```

最新の公開ベンチマークスナップショットは [パフォーマンス](./performance.md) にあります。

## トラブルシューティング

### `cargo: command not found`

Rust がインストールされ、`PATH` に入っていることを確認してください。

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### `nix: command not found`

公式インストーラーで Nix を入れ、シェルを再起動してからリポジトリへ入り直してください。

```bash
nix develop
```

### リンクエラーでビルドが失敗する

Linux ではビルド必須パッケージが必要になることがあります。

```bash
# Ubuntu / Debian
sudo apt-get install build-essential

# Fedora
sudo dnf groupinstall "Development Tools"
```

macOS では Xcode Command Line Tools を入れてください。

```bash
xcode-select --install
```

### N-API ビルドが失敗する

想定している Node.js バージョンであることを確認してください。

```bash
nix develop
node -v
vp run build:napi
```

Nix の外で Node.js を管理している場合は、`.node-version` のバージョンに合わせてください。

### `wasm-pack: command not found`

WASM ビルドタスクは `wasm-pack` が使えることを期待しています。

```bash
nix develop
vp run build:wasm
```

Nix を使っていない場合は、`wasm-pack` を手動で入れ、`rustup` 経由で `wasm32-unknown-unknown` が使えるようにしてください。

## ヘルプ

- [GitHub Issues](https://github.com/ubugeeei-prod/ox-content/issues)
- [Discussions](https://github.com/ubugeeei-prod/ox-content/discussions)
