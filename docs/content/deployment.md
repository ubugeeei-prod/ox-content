---
title: Docs Deployment
description: Deploy the Ox Content documentation directly to Cloudflare at ox-content.dev.
---

# Docs Deployment

The documentation lives at [https://ox-content.dev](https://ox-content.dev).
GitHub Actions builds and deploys it directly to Cloudflare Workers Static
Assets using Cloudflare's `cf` CLI on pushes to `main`.

The deployment includes the docs at `/`, Rust API documentation at `/api/`,
and the playground at `/playground/`. `tools/deploy/cloudflare.config.ts` declares the
`ox-content-docs` Worker and its `ox-content.dev` Custom Domain.

## Local Deployment

Install workspace dependencies and authenticate once:

```bash
vp install
vp run deploy#cf -- auth login
vp run deploy#cf -- auth whoami
```

Build and deploy from the repository root:

```bash
vp run deploy#docs
```

The task builds Rust, local npm packages (including Code Play), docs, the
playground, and Rust API documentation with `vp run build`. It assembles their
output into `dist/`, then uses Cloudflare's `@cloudflare/build-output-utils`
to write native Build Output under `tools/deploy/.cloudflare/output/v0/`.
It runs `cf deploy --prebuilt --mode production` against that output.
The `cf` package is pinned to `1.0.0-beta.9` in the deployment workspace.

Extra arguments are forwarded to `cf deploy`. Validate the full build without
publishing or needing Cloudflare credentials:

```bash
vp run deploy#docs -- --dry-run
```

Run `cf` directly through the same pinned CLI after preparing the build:

```bash
vp run deploy#cf -- deploy --prebuilt --mode production --dry-run
vp run deploy#cf -- deploy --prebuilt --mode production
```

The CLI wrapper runs in `tools/deploy`. Always use `--prebuilt --mode production`
when deploying the prepared output. Use `vp run dev:docs` to preview the docs locally.
See [Cloudflare prebuilt deployments](https://developers.cloudflare.com/cf/projects/#deploy-a-prebuilt-build).

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
[Cloudflare CLI CI instructions](https://developers.cloudflare.com/cf/ci/)
for creating the token and finding the account ID.

Set the secrets with GitHub CLI; the token prompt keeps it out of shell history:

```bash
gh secret set CLOUDFLARE_API_TOKEN --repo ubugeeei-prod/ox-content
gh secret set CLOUDFLARE_ACCOUNT_ID --repo ubugeeei-prod/ox-content
```

The workflow is `.github/workflows/deploy.yml`. Pull requests affecting deployment
configuration build and run `cf deploy --prebuilt --mode production --dry-run` without credentials.
Only pushes or manual runs on `main` publish the site. Credentials are supplied
only to the publishing step; missing secrets fail that step with setup guidance.

## Custom Domain

The `ox-content.dev` zone must be active in the same Cloudflare account used by
`cf`. The config sets `worker.domains: ["ox-content.dev"]`, so `cf` configures the
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
For another deployment target, update the Worker name and domains in
`tools/deploy/cloudflare.config.ts`, then rebuild before deploying.

Cloudflare serves directory indexes and extensionless HTML routes with
`auto-trailing-slash` handling. Missing pages use the generated `404.html` with
an HTTP 404 status.
