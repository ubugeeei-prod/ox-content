---
title: ドキュメントのデプロイ
description: Ox Content のドキュメントを Cloudflare へ直接デプロイし、ox-content.dev で公開します。
---

# ドキュメントのデプロイ

ドキュメントは [https://ox-content.dev](https://ox-content.dev) で公開します。
`main` への push を契機に、Cloudflare Workers Builds が公式 CLI の `cf` を使って
Workers Static Assets へデプロイします。GitHub Actions は認証不要の dry-run で検証します。

ドキュメントは `/`、Rust API は `/api/`、Playground は `/playground/` に配置します。
`tools/deploy/cloudflare.config.ts` に `ox-content-docs` Worker と `ox-content.dev` の Custom Domain を定義しています。

## ローカルからのデプロイ

依存関係をインストールし、一度認証します。

```bash
vp install
vp run deploy#cf -- auth login
vp run deploy#cf -- auth whoami
```

リポジトリのルートからビルドとデプロイを実行します。

```bash
vp run deploy#docs
```

このタスクは `vp run build` で Rust、Code Play を含むローカル npm パッケージ、
ドキュメント、Playground、Rust API ドキュメントをビルドします。
出力を `dist/` にまとめ、Cloudflare の `@cloudflare/build-output-utils` で
`tools/deploy/.cloudflare/output/v0/` にネイティブの Build Output を生成します。
その成果物を `cf deploy --prebuilt --mode production` でデプロイします。
deploy ワークスペースの `cf` パッケージは `1.0.0-beta.9` に固定しています。
デプロイ用スクリプトは TypeScript で、Node 26 の stable type stripping により直接実行します。
import は `.ts` を明記し、ローダーやトランスパイルは使いません。

追加の引数は `cf deploy` に転送します。公開せずにビルドと設定を検証するには、認証不要の dry-run を使います。

```bash
vp run deploy#docs -- --dry-run
```

デプロイコマンドを実行せず、同じ Build Output を準備するには次を使います。

```bash
vp run deploy#docs:build
```

成果物の準備後、同じ CLI ラッパーから `cf` を直接実行できます。

```bash
vp run deploy#cf -- deploy --prebuilt --mode production --dry-run
vp run deploy#cf -- deploy --prebuilt --mode production
```

CLI ラッパーは `tools/deploy` で実行します。準備済みの成果物をデプロイする際は
`--prebuilt --mode production` を指定してください。ローカルプレビューは `vp run dev:docs` を使います。
[Cloudflare の prebuilt デプロイ](https://developers.cloudflare.com/cf/projects/#deploy-a-prebuilt-build) も参照してください。

## Workers Builds の設定

Cloudflare のダッシュボードで `ox-content-docs` Worker を作成または選択し、
Git integration から `ubugeeei-prod/ox-content` を接続します。
[Cloudflare Workers and Pages GitHub App](https://github.com/apps/cloudflare-workers-and-pages)
を organization にインストールし、このリポジトリへのアクセスを許可してください。

ビルド設定は次の値にします。

| 設定                                | 値                                                                  |
| ----------------------------------- | ------------------------------------------------------------------- |
| Production branch                   | `main`                                                              |
| Root directory                      | リポジトリのルート (`/`)                                            |
| Build command                       | `bash tools/scripts/build-docs-on-cloudflare.sh`                    |
| Deploy command                      | `pnpm exec vp run deploy#cf -- deploy --prebuilt --mode production` |
| 本番以外の branch builds / previews | 無効                                                                |
| API token                           | Cloudflare が自動生成する既定のもの                                 |

次の **build variables** を設定します。バージョンとインストール方法の指定であり、secrets ではありません。

| 変数                      | 値       |
| ------------------------- | -------- |
| `NODE_VERSION`            | `26`     |
| `PNPM_VERSION`            | `12.1.0` |
| `SKIP_DEPENDENCY_INSTALL` | `1`      |

ビルドスクリプトはリポジトリの Rust toolchain、固定した pnpm 依存関係、
ドキュメント描画用ブラウザーをインストールします。`vp run deploy#docs:build` で
CI と同じ Build Output を生成し、別の deploy command が固定した `cf` で公開します。

デプロイ用の認証情報は Workers Builds が Cloudflare 内で自動生成して保持します。
GitHub Actions に `CLOUDFLARE_API_TOKEN` や `CLOUDFLARE_ACCOUNT_ID` の secrets を
作成・コピーする必要はなく、GitHub にトークンを登録・更新する運用も不要です。
GitHub Actions の OIDC 認証ではなく、Cloudflare の Git integration を使う構成です。
[Workers Builds の設定](https://developers.cloudflare.com/workers/ci-cd/builds/configuration/) と
[build image の設定](https://developers.cloudflare.com/workers/ci-cd/builds/build-image/) を参照してください。

`.github/workflows/deploy.yml` は、デプロイ関連の PR、`main` への push、手動実行で
認証情報を使わずビルドと `cf deploy --prebuilt --mode production --dry-run` を検証します。
本番公開は Workers Builds の production branch への push または Cloudflare 上の再実行で行います。
これらのコマンドが `main` に入ってから自動ビルドを有効にし、初回の本番ビルド成功と
公開 URL を確認してからホスティングの移行完了としてください。

## Custom Domain

`ox-content.dev` の zone は `cf` が使う account 内で active になっている必要があります。
設定の `worker.domains: ["ox-content.dev"]` により `cf` がドメインを接続し、Cloudflare が DNS と
HTTPS 証明書を管理します。別の origin server は不要です。
[Cloudflare Custom Domains](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/) を参照してください。

同じホスト名に既存の CNAME があると Custom Domain を作成できません。
初回デプロイ前に競合するレコードを確認してください。
dry-run はビルドと設定の検証であり、account の権限とドメイン登録は初回の認証付きデプロイで確認します。

## URL とアセットパス

docs ビルドの既定値は `OX_CONTENT_DOCS_BASE=/` と
`OX_CONTENT_DOCS_SITE_URL=https://ox-content.dev` です。
Playground の本番 base は `/playground/` です。
メタデータ、sitemap、feed、OG 画像、アセットはこのドメインとルート相対パスを使います。

別のビルド先には `OX_CONTENT_DOCS_BASE`、`OX_CONTENT_DOCS_SITE_URL`、
`OX_CONTENT_PLAYGROUND_BASE` を指定できます。別ドメインへのデプロイは
`tools/deploy/cloudflare.config.ts` の Worker 名と domains を変更し、再ビルドしてください。

Cloudflare は `auto-trailing-slash` でディレクトリの index と拡張子なしの HTML ルートを解決します。
存在しないページは、生成された `404.html` を HTTP 404 で返します。
