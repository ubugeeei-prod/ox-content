---
title: Release Operations
description: Release, publish, and recovery notes for maintainers.
---

# Release Operations

This page is for maintainers cutting an Ox Content release.

## Standard Release

Run the release command from your checkout. GitHub CLI must be authenticated as
someone with Maintain or Admin repository permission:

```bash
vp run release
```

The default bump is `patch`. Pass `minor`, `major`, `alpha`, `beta`, or an explicit
version when needed, for example `vp run release 3.3.0-alpha.1`.

The command prepares versions and the changelog in an isolated worktree from the
latest `origin/main`, opens a conventional `chore(release): v…` PR, and waits for
CI and release validation. If main advances, it updates the PR and revalidates.
It checks the PR author's current Maintain/Admin permission, merges the validated
head, verifies the merged tree, creates the release tag, and watches both
publishing workflows until the GitHub Release is published.

Release PRs run all native and editor targets, WASM, npm packaging and registry
name checks, and compilation of the published crate archives. Ordinary PRs only
add a lightweight release policy check to their existing CI. The
`release-validation` label opts maintainer tooling PRs into the full matrix.
Version changes are detected from manifests regardless of labels.

The first command installs a strict `Release pull requests` ruleset when absent
(requires Admin once). It requires `Release gate`, an up-to-date branch, and a PR,
with no bypass actors. Existing repository rules still apply. GitHub scopes this
rule to the target branch, so ordinary PRs also need the latest main, but do not
run the additional full release matrix.

For first-time npm packages, bootstrap the package and configure trusted publishing
before the PR can pass release validation. The offline helper
`vp run release <version> --prepare-only` updates versions and the changelog in
the current clean checkout without committing, creating a PR, tagging, or
publishing. Normal releases should use the default PR flow.

Prerelease tags publish to the matching npm dist-tag (`alpha`, for example) and
create a GitHub prerelease so they do not replace `latest`.

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

- `CARGO_PUBLISH_PACKAGES` in `tools/scripts/release-targets.ts`
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
# Generic new package, or @ox-content/code-play
node tools/scripts/bootstrap-npm-package.mjs npm/ox-content-code-play
```

The script packs the workspace package, publishes it from the laptop
(`--provenance=false`, dist-tag from the version: `alpha` for
`3.0.0-alpha.1`), then registers the GitHub Actions trusted publisher:

```bash
npm trust github @ox-content/code-play \
  --file publish.yml \
  --repo ubugeeei-prod/ox-content \
  --env npm \
  --allow-publish \
  -y
```

Bump every workspace package to the release version before packing, or the
tarball will pin its `@ox-content/*` dependencies to the previous one.
`--provenance=false` is required because provenance generation needs CI; the
package's `publishConfig` turns it on, and subsequent versions get it from the
workflow.

`npm trust` requires npm 11.15+, account 2FA, and a package that already
exists. The first trust call prompts for 2FA; later ones in the same five
minutes can skip it. The publish steps skip versions that already exist, so
the bootstrap publish is not republished.

## First-Time Crate Publishing

The crates.io job uses GitHub Actions Trusted Publishing. Trusted Publishing can
publish new versions of an existing crate, but it cannot create a brand-new
crate. If a release introduces a crate that has never existed on crates.io, the
first publish for that crate must be done manually by a maintainer with local
crates.io credentials:

```bash
cargo publish -p ox_content_new_crate
```

After the crate exists and trusted publishing is configured, resume the release
PR. The publishers skip versions that already exist.

## Recovering a Failed Publish

The command prints the release PR number when it starts. Resume it with:

```bash
vp run release --resume <pr-number>
```

If validation fails, the PR remains open and no tag is created. Fix the PR or
rerun its failed Actions jobs before resuming. The command still checks the
current author permission, main, and the exact validated head.

Publishing can fail because of registry outages or credentials even after
validation passes. Inspect the failed job and fix the external configuration,
then resume the same PR. Resume reruns failed publishing jobs against the existing
tag; the publishers skip package versions that already exist. It also waits for
both publishing workflows and verifies that the GitHub Release is not a draft.

Keep release tags immutable: do not move, delete, or recreate them. If a source or
workflow correction is needed after tagging, land the fix through a new PR and
cut a new version. Do not start concurrent retries of the same publish run.

## Documentation Deployment

After release changes land on `main`, deploy the docs site from the repository
root:

```bash
vp run deploy#docs
```

The task builds the local workspace, builds docs with the Void base path, and
then runs `vpx void@0.10.8 deploy`. Use `VOID_PROJECT` or forwarded Void CLI
flags for preview deployments.
