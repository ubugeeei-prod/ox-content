---
title: Docs Deployment
description: Deploy the Ox Content documentation site to Void.
---

# Docs Deployment

The repository exposes a dedicated workspace task for deploying the docs site to
Void:

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
6. `vpx void@0.9.0 deploy`

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
