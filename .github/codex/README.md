# Local Issue Agent

This repository can run Codex locally against GitHub Issues and open a pull request with the generated patch.

## How it works

1. Start the local watcher with `pnpm issue-agent watch`.
2. The watcher polls open GitHub Issues through `gh`.
3. When it sees a new issue, it creates a local branch from the default branch and runs `codex exec`.
4. If Codex leaves a patch, the watcher commits it, pushes the branch, opens a conventional-title PR, and waits for PR checks.
5. The watcher records processed issues under `.cache/issue-agent/state.json`.

This is intentionally local rather than a GitHub Actions workflow. Codex runs with the local user's Codex auth and local machine permissions, and GitHub only receives the resulting branch and pull request.

## Required setup

- Install and authenticate `gh`.
- Install and authenticate Codex CLI, or set the Codex auth environment expected by your local setup.
- Keep the working tree clean before starting the watcher.

## Commands

Run once for a specific issue:

```bash
pnpm issue-agent run --issue 123
```

Watch for new issues:

```bash
pnpm issue-agent watch
```

On the first `watch` run, existing open issues are marked as seen so the watcher only reacts to issues opened after it starts. To process existing open issues too, pass `--backfill`.

Useful options:

- `--repo owner/name` overrides repository detection.
- `--base main` overrides the target branch.
- `--interval 60` changes the watch polling interval in seconds.
- `--no-watch-ci` opens the PR without waiting for checks.

## Pull request titles

The script derives the PR title from the issue title when it is already conventional, for example `feat: add parser option`. `bug: ...` issue titles become `fix: ...`. Otherwise the script falls back to a conventional title inferred from labels, such as `feat: ...`, `fix: ...`, or `chore: implement issue #123`.

The script never prefixes PR titles with `[codex]`.

## Guardrails

The issue title and body are treated as untrusted prompt context. If Codex changes GitHub workflow, GitHub action, Codex prompt, or the local issue-agent script files, the script refuses to open a PR and leaves the branch for maintainer inspection.
