---
title: ドキュメントのデプロイ
description: Ox Content のドキュメントを Cloudflare へ直接デプロイし、ox-content.dev で公開します。
---

# ドキュメントのデプロイ

ドキュメントは [https://ox-content.dev](https://ox-content.dev) で公開します。
`main` への push を契機に、GitHub Actions から Cloudflare の公式 CLI である
`cf` を使い、Cloudflare Workers Static Assets へ直接デプロイします。

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

追加の引数は `cf deploy` に転送します。公開せずにビルドと設定を検証するには、認証不要の dry-run を使います。

```bash
vp run deploy#docs -- --dry-run
```

成果物の準備後、同じ CLI ラッパーから `cf` を直接実行できます。

```bash
vp run deploy#cf -- deploy --prebuilt --mode production --dry-run
vp run deploy#cf -- deploy --prebuilt --mode production
```

CLI ラッパーは `tools/deploy` で実行します。準備済みの成果物をデプロイする際は
`--prebuilt --mode production` を指定してください。ローカルプレビューは `vp run dev:docs` を使います。
[Cloudflare の prebuilt デプロイ](https://developers.cloudflare.com/cf/projects/#deploy-a-prebuilt-build) も参照してください。

## GitHub Actions の設定

デプロイ変更をマージする前に、次の repository Actions secrets を登録してください。

| Secret                  | 用途                                                       |
| ----------------------- | ---------------------------------------------------------- |
| `CLOUDFLARE_ACCOUNT_ID` | `ox-content.dev` の zone を所有する account。              |
| `CLOUDFLARE_API_TOKEN`  | Worker のデプロイと Custom Domain の管理を許可する token。 |

**Edit Cloudflare Workers** テンプレートで token を作成し、対象 account と
`ox-content.dev` の zone に範囲を限定します。**Account / Workers Scripts / Edit**、
**Zone / Workers Routes / Edit**、
**Zone / Zone / Read** の権限が必要です。
[`cf` の CI 手順](https://developers.cloudflare.com/cf/ci/) を参照してください。

GitHub CLI で登録できます。対話入力を使うため token はシェル履歴に残りません。

```bash
gh secret set CLOUDFLARE_API_TOKEN --repo ubugeeei-prod/ox-content
gh secret set CLOUDFLARE_ACCOUNT_ID --repo ubugeeei-prod/ox-content
```

ワークフローは `.github/workflows/deploy.yml` です。デプロイ設定を変更する PR では、
認証情報を渡さずにビルドと `cf deploy --prebuilt --mode production --dry-run` を実行します。
公開するのは `main` への push または `main` 上の手動実行だけです。
認証情報は公開ステップだけに渡し、secrets 未設定の場合は設定手順を示して失敗します。

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
