---
title: ドキュメントのデプロイ
description: Ox Content のドキュメントサイトを Void へデプロイします。
---

# ドキュメントのデプロイ

このリポジトリは、`main` への push で GitHub Actions からドキュメントサイトを Void へデプロイします。ワークフローは GitHub OIDC を使うので、長寿命の `VOID_TOKEN` シークレットは不要です。

ローカルデプロイでは、同じデプロイ経路が専用のワークスペースタスクとして公開されています。

```bash
vp run deploy#docs
```

このタスクはデプロイ前にローカルリポジトリからビルドするので、公開されるサイトは、レジストリにすでに公開されているものではなく、現在の Rust crate とローカル npm ワークスペースパッケージを使います。

## タスクが実行すること

`vp run deploy#docs` は `scripts/deploy-docs-to-void.mjs` を実行し、次を回します。

1. `cargo build --workspace`
2. `crates/ox_content_napi` で `napi build --release`
3. `npm/ox-content-islands` で `vp pack`
4. `npm/vite-plugin-ox-content` で `vp pack`
5. `npm/ox-content-code-play` で `vp run build`
6. `docs` で `vp build`
7. `vpx void@0.10.8 deploy`

デプロイコマンドの既定値は、このリポジトリが使う Void プロジェクトと docs 出力ディレクトリです。

| 設定                       | 既定                          | 目的                                         |
| -------------------------- | ----------------------------- | -------------------------------------------- |
| `VOID_PROJECT`             | `ox-content`                  | `void deploy --project` に渡します。         |
| `OX_CONTENT_DOCS_BASE`     | `/`                           | Void ホスト向けサイトの Vite base パスです。 |
| `OX_CONTENT_DOCS_SITE_URL` | `https://ox-content.void.app` | メタデータと OG に使う絶対サイト URL です。  |
| デプロイディレクトリ       | `docs/dist/docs`              | `void deploy --dir` に渡します。             |

Void は `https://ox-content.void.app` をルートパスでホストするので、デプロイタスクは docs の base を既定で `/` にします。その上書きなしの通常の本番 docs ビルドは、いまも `docs/vite.config.ts` で設定した GitHub Pages の base を使います。

## GitHub Actions OIDC

トークンなしデプロイのワークフローは `.github/workflows/void-deploy.yml` にあります。
`id-token: write` を付与し、GitHub Actions のシェルステップから `scripts/deploy-docs-to-void.mjs` を直接実行します。これにより `void deploy` は実行時に GitHub OIDC を短寿命の Void デプロイトークンへ交換できます。

リポジトリは一度 Void プロジェクトへ接続する必要があります。

```bash
vpx void@0.10.8 github connect ox-content \
  --repo ubugeeei-prod/ox-content \
  --branch main \
  --executor github_actions \
  --workflow .github/workflows/void-deploy.yml
```

組織向けの GitHub App がまだ入っていない場合は、先に `vpx void@0.10.8 github install` を実行してください。

## 上書き

よく使うデプロイ先には環境変数を使います。

```bash
VOID_PROJECT=ox-content-preview vp run deploy#docs
```

```bash
OX_CONTENT_DOCS_BASE=/ \
OX_CONTENT_DOCS_SITE_URL=https://ox-content.void.app \
vp run deploy#docs
```

余分な引数は `void deploy` へ転送されるので、プロジェクトやディレクトリはコマンドラインからも上書きできます。

```bash
vp run deploy#docs -- --project ox-content-preview --dir docs/dist/docs
```

## CSS とアセットパス

デプロイしたサイトで HTML は読み込めるのに CSS やクライアントアセットが欠けている場合は、まず base パスを確認してください。Void へのデプロイは次でビルドします。

```bash
OX_CONTENT_DOCS_BASE=/ vp run deploy#docs
```

生成 HTML は `/ox-content/assets/index.css` ではなく、`/assets/index.css` のようなルート相対アセットを参照する必要があります。
