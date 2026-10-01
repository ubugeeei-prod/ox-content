---
title: Docs Deployment
description: Deploy the Ox Content documentation directly to Cloudflare at ox-content.dev.
---

# Docs Deployment

The documentation lives at [https://ox-content.dev](https://ox-content.dev).
Cloudflare Workers Builds builds and deploys it to Workers Static Assets using
the `cf` CLI on pushes to `main`. GitHub Actions validates the deployment with
an unauthenticated dry-run and does not publish the site.

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

To prepare the same Build Output without running a deployment command:

```bash
vp run deploy#docs:build
```

Run `cf` directly through the same pinned CLI after preparing the build:

```bash
vp run deploy#cf -- deploy --prebuilt --mode production --dry-run
vp run deploy#cf -- deploy --prebuilt --mode production
```

The CLI wrapper runs in `tools/deploy`. Always use `--prebuilt --mode production`
when deploying the prepared output. Use `vp run dev:docs` to preview the docs locally.
See [Cloudflare prebuilt deployments](https://developers.cloudflare.com/cf/projects/#deploy-a-prebuilt-build).

## Workers Builds Setup

In the Cloudflare dashboard, create or select the `ox-content-docs` Worker and
connect `ubugeeei-prod/ox-content` through its Git integration. Install the
[Cloudflare Workers and Pages GitHub App](https://github.com/apps/cloudflare-workers-and-pages)
for the organization with access to this repository.

Use these build settings:

| Setting                                 | Value                                                               |
| --------------------------------------- | ------------------------------------------------------------------- |
| Production branch                       | `main`                                                              |
| Root directory                          | Repository root (`/`)                                               |
| Build command                           | `bash tools/scripts/build-docs-on-cloudflare.sh`                    |
| Deploy command                          | `pnpm exec vp run deploy#cf -- deploy --prebuilt --mode production` |
| Non-production branch builds / previews | Disabled                                                            |
| API token                               | Use Cloudflare's automatically generated default                    |

Set these **build variables**, which are ordinary version and installation
settings rather than secrets:

| Variable                  | Value    |
| ------------------------- | -------- |
| `NODE_VERSION`            | `26`     |
| `PNPM_VERSION`            | `12.1.0` |
| `SKIP_DEPENDENCY_INSTALL` | `1`      |

The build script installs the repository's Rust toolchain, the frozen pnpm
dependencies, and the browsers needed to render documentation. It runs
`vp run deploy#docs:build` to assemble the same Build Output validated in CI.
The separate deploy command uses the pinned `cf` CLI without rebuilding.

Workers Builds automatically generates and stores its deployment credential
inside Cloudflare. No `CLOUDFLARE_API_TOKEN` or `CLOUDFLARE_ACCOUNT_ID` GitHub
Actions secrets need to be created, copied, or rotated by repository maintainers.
This is Cloudflare Git integration, not GitHub Actions OIDC authentication.
See [Workers Builds configuration](https://developers.cloudflare.com/workers/ci-cd/builds/configuration/)
and [build image settings](https://developers.cloudflare.com/workers/ci-cd/builds/build-image/).

The `.github/workflows/deploy.yml` workflow builds and runs
`cf deploy --prebuilt --mode production --dry-run` without credentials on
deployment-related pull requests, pushes to `main`, and manual runs. Production
publishing happens only in Workers Builds after a push to its production branch
or a manual retry in Cloudflare. Enable automatic builds once these commands
are available on `main`, and check the first successful production build and
the public URLs before considering the hosting migration complete.

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
