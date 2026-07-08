---
title: Docs Deployment
description: Deploy the Ox Content documentation site to Void.
---

# Docs Deployment

The repository deploys the docs site to Void from GitHub Actions on pushes to
`main`. The workflow uses GitHub OIDC, so it does not require a long-lived
`VOID_TOKEN` secret.

For local deployments, the same deploy path is exposed as a dedicated workspace
task:

```bash
vp run deploy#docs
```

The task builds from the local repository before deploying, so the published
site uses the current Rust crates and local npm workspace packages rather than
whatever is already published to the registry.

## What the Task Runs

`vp run deploy#docs` executes `scripts/deploy-docs-to-void.mjs`, which runs:

1. `cargo build --workspace`
2. `napi build --release` in `crates/ox_content_napi`
3. `vp pack` in `npm/ox-content-islands`
4. `vp pack` in `npm/vite-plugin-ox-content`
5. `vp build` in `docs`
6. `vpx void@0.10.8 deploy`

The deploy command defaults to the Void project and docs output directory used
by this repository.

| Setting                    | Default                       | Purpose                                     |
| -------------------------- | ----------------------------- | ------------------------------------------- |
| `VOID_PROJECT`             | `ox-content`                  | Passed to `void deploy --project`.          |
| `OX_CONTENT_DOCS_BASE`     | `/`                           | Vite base path for the Void-hosted site.    |
| `OX_CONTENT_DOCS_SITE_URL` | `https://ox-content.void.app` | Absolute site URL used for metadata and OG. |
| Deploy directory           | `docs/dist/docs`              | Passed to `void deploy --dir`.              |

Void hosts `https://ox-content.void.app` at the root path, so the deploy task
sets the docs base to `/` by default. A normal production docs build without
that override still uses the GitHub Pages base configured in `docs/vite.config.ts`.

## GitHub Actions OIDC

The tokenless deploy workflow lives at `.github/workflows/void-deploy.yml`.
It grants `id-token: write` and runs `vp run deploy#docs`, which lets
`void deploy` exchange GitHub OIDC for a short-lived Void deploy token at run
time.

The repository must be connected to the Void project once:

```bash
vpx void@0.10.8 github connect ox-content \
  --repo ubugeeei-prod/ox-content \
  --branch main \
  --executor github_actions \
  --workflow .github/workflows/void-deploy.yml
```

If the GitHub App is not installed for the organization yet, run
`vpx void@0.10.8 github install` first.

## Overrides

Use environment variables for common deployment targets:

```bash
VOID_PROJECT=ox-content-preview vp run deploy#docs
```

```bash
OX_CONTENT_DOCS_BASE=/ \
OX_CONTENT_DOCS_SITE_URL=https://ox-content.void.app \
vp run deploy#docs
```

Extra arguments are forwarded to `void deploy`, so the project or directory can
also be overridden from the command line:

```bash
vp run deploy#docs -- --project ox-content-preview --dir docs/dist/docs
```

## CSS and Asset Paths

If the deployed site loads HTML but misses CSS or client assets, check the base
path first. Void deployments should build with:

```bash
OX_CONTENT_DOCS_BASE=/ vp run deploy#docs
```

The generated HTML should reference root-relative assets such as
`/assets/index.css`, not `/ox-content/assets/index.css`.
