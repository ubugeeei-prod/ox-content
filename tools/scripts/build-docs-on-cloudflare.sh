#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

# Workers Builds provides Node.js and pnpm; the repository also needs Rust.
export PATH="$HOME/.cargo/bin:$PWD/node_modules/.bin:$PATH"
if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -fsS https://sh.rustup.rs |
    sh -s -- -y --profile minimal --default-toolchain none --no-modify-path
fi
rustup show active-toolchain

pnpm install --frozen-lockfile
vp exec --filter @ox-content/vite-plugin -- playwright install chromium
vp exec --filter @ox-content/vite-plugin -- puppeteer browsers install chrome-headless-shell
vp run deploy#docs:build
