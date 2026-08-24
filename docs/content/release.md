---
title: Release Operations
description: Release, publish, and recovery notes for maintainers.
---

# Release Operations

This page is for maintainers cutting an Ox Content release.

## Standard Release

Run releases from a clean `main` checkout:

```bash
git status --short
vp run release -- patch
```

The release script updates package versions, Cargo workspace versions, docs
snippets, and the changelog. It then creates a conventional release commit and
an annotated `v*` tag. Pushing the tag starts `.github/workflows/publish.yml`.

The publish workflow handles:

1. N-API native binding builds for the supported platforms
2. `@ox-content/napi` and binding package publishing to npm
3. other npm package publishing
4. Rust crate publishing to crates.io
5. GitHub Release creation

Most publish steps are idempotent. Before publishing, they check whether the
same package version already exists and skip it when present. That makes it
safe to re-run a failed release after some packages were already published.

## Crates.io Publish Order

Rust crates must be published in dependency order. Keep both of these lists in
sync when adding a crate that should ship to crates.io:

- `CARGO_PUBLISH_PACKAGES` in `scripts/release.ts`
- `publish_crate ...` calls in `.github/workflows/publish.yml`

The release script verifies that every crate listed in
`CARGO_PUBLISH_PACKAGES` also has a publish target in the workflow. The workflow
order still matters because crates.io must see each dependency before Cargo can
package a dependent crate.

## npm Authentication

The npm jobs publish through GitHub Actions Trusted Publishing. There is no npm
token in the repository's secrets: `id-token: write` lets the job mint an OIDC
token, npm exchanges it for a short-lived publish credential, and provenance is
attested automatically on that path.

Each package carries its own trusted publisher entry on npmjs.com, naming this
repository, `.github/workflows/publish.yml`, and the `npm` environment. All
three are part of the identity, so renaming the workflow file or the environment
breaks publishing until every entry is updated to match.

Entries are needed for the workspace packages (`@ox-content/napi`,
`@ox-content/islands`, `@ox-content/code-play`, `@ox-content/vite-plugin`,
`@ox-content/unplugin`, the four
`@ox-content/vite-plugin-{vue,react,svelte,solid}` integrations, and
`@ox-content/wasm`) and for each `@ox-content/napi-*` platform binding package
the N-API build publishes.

On npmjs.com the GitHub Actions trusted publisher must match this identity
exactly:

- Organization or user: `ubugeeei-prod`
- Repository: `ox-content`
- Workflow filename: `publish.yml`
- Environment name: `npm`

## First-Time npm Publishing

Trusted publishing cannot create a package that does not exist yet: the
publisher entry is configured on the package's settings page, so the package has
to be there first. Same shape as the crates.io restriction below.

A release that introduces a new npm package therefore needs one manual publish
by a maintainer with local npm credentials, before the tag is pushed:

```bash
# Generic new package
corepack pnpm --filter @ox-content/vite-plugin-new build
cd npm/vite-plugin-ox-content-new
corepack pnpm pack --pack-destination /tmp
npm publish /tmp/ox-content-vite-plugin-new-<version>.tgz --access public --provenance=false

# @ox-content/code-play bootstrap (package does not exist on npm yet)
node scripts/bootstrap-npm-package.mjs npm/ox-content-code-play
```

Bump every workspace package to the release version before packing, or the
tarball will pin its `@ox-content/*` dependencies to the previous one.
`--provenance=false` is required because provenance generation needs CI; the
package's `publishConfig` turns it on, and subsequent versions get it from the
workflow.

Then add the trusted publisher entry on npmjs.com and push the tag. The publish
steps skip versions that already exist, so the bootstrap publish is not
republished.

## First-Time Crate Publishing

The crates.io job uses GitHub Actions Trusted Publishing. Trusted Publishing can
publish new versions of an existing crate, but it cannot create a brand-new
crate. If a release introduces a crate that has never existed on crates.io, the
first publish for that crate must be done manually by a maintainer with local
crates.io credentials:

```bash
cargo publish -p ox_content_new_crate
```

After the crate exists, push or re-run the tag workflow. The workflow will skip
already-published crates and continue with the remaining packages.

## Recovering a Failed Publish

If `.github/workflows/publish.yml` fails:

1. Inspect the failing job log in GitHub Actions.
2. Check which package versions already exist:

   ```bash
   curl -fsSL https://crates.io/api/v1/crates/ox_content_parser/2.75.0 >/dev/null
   npm view @ox-content/vite-plugin@2.75.0 version
   ```

3. Fix the workflow or publish any first-time crates manually when required.
4. Move the release tag to the fixed commit and push the tag again:

   ```bash
   git tag -f -a v2.75.0 -m "Release v2.75.0" HEAD
   git push --force origin refs/tags/v2.75.0
   ```

5. If GitHub does not start a new workflow for a tag-object-only update, delete
   and recreate the remote tag:

   ```bash
   git push origin :refs/tags/v2.75.0
   git push origin refs/tags/v2.75.0
   ```

6. Watch the new `Publish` run until it succeeds.

Cancel duplicate publish runs when more than one tag push starts the workflow.
Only one run should be allowed to publish at a time.

## Documentation Deployment

After release changes land on `main`, deploy the docs site from the repository
root:

```bash
vp run deploy#docs
```

The task builds the local workspace, builds docs with the Void base path, and
then runs `vpx void@0.10.8 deploy`. Use `VOID_PROJECT` or forwarded Void CLI
flags for preview deployments.
