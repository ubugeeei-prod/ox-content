# Issue Agent

This repository can run Codex from GitHub Issues and open a pull request with the generated patch.

## How it works

1. A new or reopened issue triggers `.github/workflows/issue-agent.yml`.
2. The workflow checks out the default branch, prepares the repository, and runs `openai/codex-action`.
3. Codex writes a local patch only. The OpenAI API key is not passed to the PR creation job.
4. The PR job applies the patch, creates a branch, opens a pull request, dispatches `CI`, and waits for the result.
5. The workflow comments on the issue with the PR URL and CI result.

The workflow allows all GitHub users to trigger Codex from issues. It still blocks generated patches that modify GitHub workflow, GitHub action, or Codex automation files, because the PR job dispatches CI for the generated branch.

## Required setup

- Create a repository or organization secret named `OPENAI_API_KEY`.
- In repository Actions settings, allow GitHub Actions to create pull requests.

## Pull request titles

The workflow derives the PR title from the issue title when it is already conventional, for example `feat: add parser option`. `bug: ...` issue titles become `fix: ...`. Otherwise the workflow falls back to a conventional title inferred from labels, such as `feat: ...`, `fix: ...`, or `chore: implement issue #123`.

The workflow never prefixes PR titles with `[codex]`.

## CI

Pull requests created with the default `GITHUB_TOKEN` may not automatically trigger `pull_request` workflows. To make CI verification deterministic, `CI` also supports `workflow_dispatch`, and the issue agent manually dispatches `ci.yml` for the generated branch before reporting the result.
