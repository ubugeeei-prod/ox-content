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

Entries are needed for the workspace packages (`@ox-content/cli`, `@ox-content/napi`,
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
to be registered first. Registration can precede its first public version.

Use npm staged publishing to reserve the package name with local maintainer
authentication before pushing the release tag. The registration candidate stays
unpublished; GitHub Actions publishes the first public version through OIDC.

```bash
# Generic new package, or @ox-content/code-play
node tools/scripts/bootstrap-npm-package.mjs npm/ox-content-code-play

# Standalone @ox-content/cli package (installed command: oxct)
vp node tools/scripts/bootstrap-npm-package.mjs npm/oxct --pack-only
# After reviewing the tarball, authenticate npm and bootstrap the package:
vp node tools/scripts/bootstrap-npm-package.mjs npm/oxct
```

The script packs the workspace package, stages a separate registration prerelease
(`3.3.0-registration.0` for release `3.3.0`), then registers the GitHub Actions
trusted publisher. Keeping a separate version avoids reserving the release's
version number in the staged-version index.

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
The local staging command uses `--provenance=false` because attestation requires
CI. The first public version gets provenance from the OIDC publishing workflow.
Release validation uses `npm access get status` to require public registration;
`npm view` returns 404 until a version has actually been published.

`npm trust` requires npm 11.15+, account 2FA, and a package that already
exists. The first trust call prompts for 2FA; later ones in the same five
minutes can skip it. After confirming the trusted publisher, the maintainer can
remove the unused staged candidate with `npm stage reject <stage-id>` and log
out of the temporary local session. Do not approve the registration candidate.
See [npm staged publishing](https://docs.npmjs.com/cli/v11/commands/npm-stage/).

## Open VSX Trusted Publishing

The Open VSX publishing job uses GitHub Actions OIDC with `ovsx --trusted-publishing`.
It requires `id-token: write` and the `open-vsx` environment; no `OVSX_PAT` secret
is needed. A publishing run fails if its OIDC credentials or registry registration
are unavailable. Packaging-only dispatches still build without publishing.

Open VSX currently requires the extension to have an active published version
before a namespace owner can register its trusted publisher. Complete that
one-time registration in Open VSX Settings → Trusted Publishers with:

- Namespace: `ubugeeei`
- Extension: `vscode-ox-content`
- Provider: GitHub Actions
- Organization: `ubugeeei-prod`
- Repository: `ox-content`
- Workflow: `publish-editors.yml`
- Environment: `open-vsx`

The namespace owner must also sign the Open VSX Publisher Agreement. See the
[Open VSX trusted publishing guide](https://github.com/eclipse-openvsx/openvsx/wiki/Trusted-Publishing)
for the initial publication and registration requirements.

### Bootstrap and recover Open VSX

`No trusted publisher matches the presented token` is a registration failure.
The publisher exits immediately with setup instructions; retrying cannot create
the extension or register its trusted publisher. Temporary registry failures
still use the normal retry backoff.

Complete the following once as the maintainer:

1. Sign in to [Open VSX settings](https://open-vsx.org/user-settings/profile),
   connect the Eclipse account, and review and sign the Publisher Agreement.
2. Check that the account owns the `ubugeeei` namespace. A namespace contributor
   can publish but cannot register a trusted publisher. If necessary, follow
   [namespace ownership instructions](https://github.com/eclipse-openvsx/openvsx/wiki/Namespace-Access#how-to-claim-a-namespace).
3. Download `vscode-vsix-linux-x64` from the existing release's **Publish editor
   extensions** run. Extract and inspect the VSIX: its publisher must be
   `ubugeeei`, name `vscode-ox-content`, version the intended release, and target
   `linux-x64`. Keep the release tag and the existing artifacts intact.
4. Generate a temporary token in [Access Tokens](https://open-vsx.org/user-settings/tokens)
   and use it only in the local shell for this initial upload:

   ```bash
   read -rs 'OVSX_PAT?Temporary Open VSX token: '; echo
   export OVSX_PAT
   vp exec -- pnpm dlx --allow-build @vscode/vsce-sign --allow-build keytar ovsx@1.2.0 \
     publish /path/to/vscode-ox-content-linux-x64.vsix --skip-duplicate
   unset OVSX_PAT
   ```

   This prompt uses zsh. Do not paste the token into chat, commit it, or add it as
   a CI secret. Revoke the temporary token after the initial upload.

5. In [Trusted Publishers](https://open-vsx.org/user-settings/trusted-publishers),
   register `vscode-ox-content` with the identity listed above.
6. Rerun only the failed **Publish Open VSX** job from the same release run.
   Verify the exact version in the public API for `darwin-arm64`, `darwin-x64`,
   `linux-x64`, and `win32-x64`, using
   `https://open-vsx.org/api/ubugeeei/vscode-ox-content/<target>/<version>`.

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

The task builds the local workspace and deploys the docs, Rust API reference,
and playground to `https://ox-content.dev` using the `cf` CLI. Run
`vp run deploy#docs -- --dry-run` to validate without publishing. See
[Docs Deployment](./deployment.md) for authentication and GitHub Actions setup.
