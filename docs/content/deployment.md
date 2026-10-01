---
title: Docs Deployment
description: Deploy the Ox Content documentation directly to Cloudflare at ox-content.dev.
---

# Docs Deployment

The documentation lives at [https://ox-content.dev](https://ox-content.dev).
GitHub Actions builds and deploys it directly to Cloudflare Workers Static
Assets using Cloudflare's Wrangler CLI on pushes to `main`.

The deployment includes the docs at `/`, Rust API documentation at `/api/`,
and the playground at `/playground/`. `wrangler.jsonc` declares the
`ox-content-docs` Worker and its `ox-content.dev` Custom Domain.

## Local Deployment

Install workspace dependencies and authenticate once:

```bash
vp install
vp run deploy#cf login
vp run deploy#cf whoami
```

Build and deploy from the repository root:

```bash
vp run deploy#docs
```

The task builds Rust, local npm packages (including Code Play), docs, the
playground, and Rust API documentation with `vp run build`. It assembles their
output into `dist/` and runs `wrangler deploy`. Wrangler is pinned to `4.145.0`;
the CLI wrapper supplies the build approvals required by pnpm 12.

Extra arguments are forwarded to Wrangler. Validate the full build without
publishing or needing Cloudflare credentials:

```bash
vp run deploy#docs -- --dry-run
```

Run Wrangler directly through the same pinned CLI:

```bash
vp run deploy#cf -- dev --local
vp run deploy#cf -- deploy --dry-run
```

These commands read `wrangler.jsonc` from the repository root. Build the
`dist/` bundle with the docs task before starting local preview.

## GitHub Actions Setup

Configure these repository Actions secrets before merging the deployment change:

| Secret                  | Purpose                                                            |
| ----------------------- | ------------------------------------------------------------------ |
| `CLOUDFLARE_ACCOUNT_ID` | The account that owns the `ox-content.dev` zone.                   |
| `CLOUDFLARE_API_TOKEN`  | A token allowed to deploy the Worker and manage its Custom Domain. |

Create a token using the **Edit Cloudflare Workers** template, scoped to the
correct account and the `ox-content.dev` zone. It needs **Account / Workers
Scripts / Edit**, **Zone / Workers Routes / Edit**, and **Zone / Zone / Read**
permissions. Follow the
[Cloudflare authentication instructions](https://developers.cloudflare.com/workers/ci-cd/external-cicd/github-actions/)
for creating the token and finding the account ID.

Set the secrets with GitHub CLI; the token prompt keeps it out of shell history:

```bash
gh secret set CLOUDFLARE_API_TOKEN --repo ubugeeei-prod/ox-content
gh secret set CLOUDFLARE_ACCOUNT_ID --repo ubugeeei-prod/ox-content
```

The workflow is `.github/workflows/deploy.yml`. Pull requests affecting deployment
configuration build and run `wrangler deploy --dry-run` without credentials.
Only pushes or manual runs on `main` publish the site. Credentials are supplied
only to the publishing step; missing secrets fail that step with setup guidance.

## Custom Domain

The `ox-content.dev` zone must be active in the same Cloudflare account used by
Wrangler. The config sets `custom_domain: true`, so Wrangler configures the
domain and Cloudflare manages DNS and HTTPS certificates. No separate origin
server is needed. See [Cloudflare Custom Domains](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/).

An existing CNAME for the same hostname prevents Custom Domain creation. Check
for conflicting records before the first deployment. A successful dry-run
validates the bundle and config; the first authenticated deployment verifies
account access and domain registration.

## URLs and Asset Paths

Docs builds default to `OX_CONTENT_DOCS_BASE=/` and
`OX_CONTENT_DOCS_SITE_URL=https://ox-content.dev`. The playground production
base defaults to `/playground/`. Generated metadata, sitemaps, feeds, OG images,
and client assets therefore use the custom domain and root-relative paths.

`OX_CONTENT_DOCS_BASE`, `OX_CONTENT_DOCS_SITE_URL`, and
`OX_CONTENT_PLAYGROUND_BASE` remain available for alternate build targets.
Use a separate Wrangler config with `--config` when deploying to another domain.

Cloudflare serves directory indexes and extensionless HTML routes with
`auto-trailing-slash` handling. Missing pages use the generated `404.html` with
an HTTP 404 status.
