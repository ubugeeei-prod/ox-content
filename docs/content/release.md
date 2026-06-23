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
then runs `vpx void@0.9.0 deploy`. Use `VOID_PROJECT` or forwarded Void CLI
flags for preview deployments.
