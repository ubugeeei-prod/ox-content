# Releasing

Run `vp run release` (patch by default), or `vp run release minor`, `major`,
`alpha`, `beta`, or an explicit version such as `3.3.0`.
GitHub CLI must be authenticated as a repository maintainer (Maintain or Admin).

The command prepares versions and the changelog in a separate worktree based on
the latest `origin/main`, creates a `chore(release): v…` PR on `release/v…`, and
waits for GitHub Actions. It does not change your current checkout.

Only release PRs run the additional release validation: all seven native binding
targets, all four VSIX targets, Zed, WASM, every npm tarball, registry package name
preflight, and compilation of the published crate archives. These use the same
native/editor build workflows and NAPI packaging action as publication. Ordinary
PRs retain their existing CI and only add a lightweight release policy check.
The `release-validation` label explicitly opts a tooling PR into the full matrix.
Version changes are detected from manifests, so removing labels cannot skip them.

If main advances, the command updates the PR and waits for the new checks. It
checks the PR author's current repository role and requires successful CI and
release validation for the exact head. GitHub also enforces an up-to-date branch
at merge time. After merging, the command verifies that the merged tree matches
the tested tree, creates only that release's tag, and waits for both publishing
workflows and the GitHub Release.

On failure the command exits unsuccessfully with a PR or Actions URL. Validation
failures leave the PR open and create no tag. Fix the PR or rerun the failed
Actions jobs, then use `vp run release --resume <pr-number>`. Publication can still
fail because of registry outages or credentials; resume reruns failed publishing
jobs against the same tag. It never moves or deletes an existing tag. Optional
Open VSX/Zed publication still follows the existing credential configuration.

## Repository setup

The first release command installs a `Release pull requests` ruleset if absent
(requires Admin once). It requires `Release gate` from GitHub Actions, strict
up-to-date checks, a pull request, and no bypass actors. Existing repository rules
remain in effect. A missing or weakened existing rule stops the command; it is
never silently bypassed. Install this workflow on main before the first release.

GitHub scopes strict status-check rules to the target branch, so the up-to-date
requirement also applies to ordinary PRs into main. It does not enable the full
release matrix on those PRs. Rule changes require repository administration.

`vp run release <version> --prepare-only` is an offline helper that changes
versions/changelog in the current clean checkout. It does not create a PR, merge,
tag, or publish. Normal releases should use the default PR flow.
