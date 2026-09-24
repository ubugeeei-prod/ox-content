---
title: ドキュメントのデプロイ
description: Ox Content のドキュメントサイトを Void へデプロイします。
---

# ドキュメントのデプロイ

このリポジトリは、`main` への push で GitHub Actions から Void を使ってドキュメントサイトをデプロイします。`void deploy --platform cloudflare` が静的サイトを、静的アセット付きの Worker としてプロジェクトの Cloudflare アカウントへ直接アップロードします。

ローカルデプロイでは、同じデプロイ経路が専用のワークスペースタスクとして公開されています。

```bash
vp run deploy#docs
```

このタスクはデプロイ前にローカルリポジトリからビルドするので、公開されるサイトは、レジストリにすでに公開されているものではなく、現在の Rust crate とローカル npm ワークスペースパッケージを使います。

## タスクが実行すること

`vp run deploy#docs` は `tools/scripts/deploy-docs-to-void.mjs` を実行し、次を回します。

1. `cargo build --workspace`
2. `crates/ox_content_napi` で `napi build --release`
3. `npm/ox-content-islands` で `vp pack`
4. `npm/vite-plugin-ox-content` で `vp pack`
5. `npm/ox-content-code-play` で `vp run build`
6. `docs` で `vp build`
7. `tools/deploy` で `void deploy --platform cloudflare`

`void` は `tools/deploy` の devDependency として固定されており、同じディレクトリの `wrangler.jsonc` が Worker 名 `ox-content` を指定します。Cloudflare アカウントは `CLOUDFLARE_ACCOUNT_ID` から読み込みます。

| 設定                       | 既定                          | 目的                                         |
| -------------------------- | ----------------------------- | -------------------------------------------- |
| `CLOUDFLARE_ACCOUNT_ID`    | なし                          | Worker をホストする Cloudflare アカウントです。 |
| `CLOUDFLARE_API_TOKEN`     | なし（ローカルではブラウザログイン） | 非対話デプロイで使う API トークンです。 |
| `OX_CONTENT_DOCS_BASE`     | `/`                           | Void ホスト向けサイトの Vite base パスです。 |
| `OX_CONTENT_DOCS_SITE_URL` | `https://ox-content.void.app` | メタデータと OG に使う絶対サイト URL です。  |
| デプロイディレクトリ       | `docs/dist/docs`              | `void deploy --dir` に渡します。             |

Void は `https://ox-content.void.app` をルートパスでホストするので、デプロイタスクは docs の base を既定で `/` にします。その上書きなしの通常の本番 docs ビルドは、いまも `docs/vite.config.ts` で設定した GitHub Pages の base を使います。

## GitHub Actions

デプロイのワークフローは `.github/workflows/void-deploy.yml` にあり、GitHub Actions のシェルステップから `tools/scripts/deploy-docs-to-void.mjs` を直接実行します。リポジトリには次の設定が必要です。

| 名前                       | 種類     | 目的                                                           |
| -------------------------- | -------- | -------------------------------------------------------------- |
| `CLOUDFLARE_API_TOKEN`     | Secret   | アカウントに対する Workers Scripts: Edit を持つ API トークン。 |
| `CLOUDFLARE_ACCOUNT_ID`    | Secret   | `ox-content` Worker をホストする Cloudflare アカウント。       |
| `OX_CONTENT_DOCS_SITE_URL` | Variable | 任意。ビルドで使う絶対サイト URL を上書きします。              |

Cloudflare のシークレットはデプロイステップにだけ渡されます。

## 上書き

よく使うデプロイ先には環境変数を使います。

```bash
OX_CONTENT_DOCS_BASE=/ \
OX_CONTENT_DOCS_SITE_URL=https://ox-content.void.app \
vp run deploy#docs
```

余分な引数は `void deploy` へ転送されるので、ディレクトリはコマンドラインからも上書きできます。

```bash
vp run deploy#docs -- --dir docs/dist/docs --debug
```

## CSS とアセットパス

デプロイしたサイトで HTML は読み込めるのに CSS やクライアントアセットが欠けている場合は、まず base パスを確認してください。Void へのデプロイは次でビルドします。

```bash
OX_CONTENT_DOCS_BASE=/ vp run deploy#docs
```

生成 HTML は `/ox-content/assets/index.css` ではなく、`/assets/index.css` のようなルート相対アセットを参照する必要があります。
