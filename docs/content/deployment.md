---
title: Docs Deployment
description: Deploy the Ox Content documentation site to Void.
---

# Docs Deployment

The repository deploys the docs site with Void from GitHub Actions on pushes to
`main`. `void deploy --platform cloudflare` uploads the static site directly to
the project's Cloudflare account as a Worker with static assets.

For local deployments, the same deploy path is exposed as a dedicated workspace
task:

```bash
vp run deploy#docs
```

The task builds from the local repository before deploying, so the published
site uses the current Rust crates and local npm workspace packages rather than
whatever is already published to the registry.

## What the Task Runs

`vp run deploy#docs` executes `tools/scripts/deploy-docs-to-void.mjs`, which runs:

1. `cargo build --workspace`
2. `napi build --release` in `crates/ox_content_napi`
3. `vp pack` in `npm/ox-content-islands`
4. `vp pack` in `npm/vite-plugin-ox-content`
5. `vp run build` in `npm/ox-content-code-play`
6. `vp build` in `docs`
7. `void deploy --platform cloudflare` in `tools/deploy`

`void` is pinned as a devDependency of `tools/deploy`, next to the
`wrangler.jsonc` that names the `ox-content` Worker. The Cloudflare account is
read from `CLOUDFLARE_ACCOUNT_ID`.

| Setting                    | Default                       | Purpose                                     |
| -------------------------- | ----------------------------- | ------------------------------------------- |
| `CLOUDFLARE_ACCOUNT_ID`    | none                          | Cloudflare account that hosts the Worker.   |
| `CLOUDFLARE_API_TOKEN`     | none (browser login locally)  | API token used for non-interactive deploys. |
| `OX_CONTENT_DOCS_BASE`     | `/`                           | Vite base path for the Void-hosted site.    |
| `OX_CONTENT_DOCS_SITE_URL` | `https://ox-content.void.app` | Absolute site URL used for metadata and OG. |
| Deploy directory           | `docs/dist/docs`              | Passed to `void deploy --dir`.              |

Void hosts `https://ox-content.void.app` at the root path, so the deploy task
sets the docs base to `/` by default. A normal production docs build without
that override still uses the GitHub Pages base configured in `docs/vite.config.ts`.

## GitHub Actions

The deploy workflow lives at `.github/workflows/void-deploy.yml` and runs
`tools/scripts/deploy-docs-to-void.mjs` directly from the GitHub Actions shell
step. It needs these repository settings:

| Name                       | Kind     | Purpose                                                  |
| -------------------------- | -------- | -------------------------------------------------------- |
| `CLOUDFLARE_API_TOKEN`     | Secret   | API token with Workers Scripts: Edit on the account.     |
| `CLOUDFLARE_ACCOUNT_ID`    | Secret   | Cloudflare account that hosts the `ox-content` Worker.   |
| `OX_CONTENT_DOCS_SITE_URL` | Variable | Optional. Overrides the absolute site URL for the build. |

The Cloudflare secrets are only exposed to the deploy step.

## Overrides

Use environment variables for common deployment targets:

```bash
OX_CONTENT_DOCS_BASE=/ \
OX_CONTENT_DOCS_SITE_URL=https://ox-content.void.app \
vp run deploy#docs
```

Extra arguments are forwarded to `void deploy`, so the directory can also be
overridden from the command line:

```bash
vp run deploy#docs -- --dir docs/dist/docs --debug
```

## CSS and Asset Paths

If the deployed site loads HTML but misses CSS or client assets, check the base
path first. Void deployments should build with:

```bash
OX_CONTENT_DOCS_BASE=/ vp run deploy#docs
```

The generated HTML should reference root-relative assets such as
`/assets/index.css`, not `/ox-content/assets/index.css`.
