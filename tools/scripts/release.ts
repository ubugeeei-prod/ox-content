#!/usr/bin/env node
// Usage: vp run release [patch|minor|major|alpha|beta|x.y.z] | --resume <pr-number>

import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { bumpVersion, prepareRelease } from "./release-prepare.ts";
import { releaseVersion, VERSION_PATTERN } from "./release-policy.ts";
import {
  api,
  authorPermission,
  ensureReleaseProtection,
  latestRun,
  mergeRelease,
  requireReleasePr,
  run,
  runPassed,
  watchPublication,
  type PullRequest,
} from "./release-github.ts";

const root = resolve(import.meta.dirname, "../..");

function createReleasePr(repo: string, input: string): number {
  run("git", ["fetch", "origin", "main", "--tags"], root);
  const current = JSON.parse(
    run("git", ["show", "origin/main:crates/ox_content_napi/package.json"], root),
  ).version;
  const bumps = ["patch", "minor", "major", "alpha", "beta"] as const;
  const version = bumps.includes(input as (typeof bumps)[number])
    ? bumpVersion(current, input as (typeof bumps)[number])
    : input;
  if (!VERSION_PATTERN.test(version)) throw new Error(`Invalid version: ${version}`);
  const branch = `release/v${version}`;
  const existing = api<PullRequest[]>(
    repo,
    `pulls?state=open&base=main&head=${encodeURIComponent(`${repo.split("/")[0]}:${branch}`)}`,
  );
  if (existing.length) return existing[0].number;
  if (current === version)
    throw new Error(`main is already v${version}; use --resume <pr-number>.`);
  if (run("git", ["ls-remote", "--tags", "origin", `refs/tags/v${version}`], root)) {
    throw new Error(`v${version} already exists; use --resume <pr-number>.`);
  }
  const temp = mkdtempSync(join(tmpdir(), "ox-content-release-"));
  const worktree = join(temp, "checkout");
  let added = false;
  try {
    run("git", ["worktree", "add", "-b", branch, worktree, "origin/main"], root);
    added = true;
    // Execute the preparation implementation from this command, against the isolated checkout.
    run(
      process.execPath,
      [join(worktree, "tools/scripts/release.ts"), version, "--prepare-only"],
      worktree,
    );
    run("git", ["add", "-A"], worktree);
    run("git", ["commit", "-m", `chore(release): v${version}`], worktree);
    run("git", ["push", "--set-upstream", "origin", branch], worktree);
    const body = join(temp, "body.md");
    writeFileSync(
      body,
      `Release v${version}.\n\nThe release command waits for the full release validation and CI, updates this PR if main advances, and merges only with strict Release gate protection. The tag is created after the merge.\n`,
    );
    const url = run(
      "gh",
      [
        "pr",
        "create",
        "--repo",
        repo,
        "--base",
        "main",
        "--head",
        branch,
        "--title",
        `chore(release): v${version}`,
        "--body-file",
        body,
      ],
      worktree,
    );
    console.log(url);
    return Number(url.split("/").at(-1));
  } finally {
    if (added && !run("git", ["status", "--porcelain"], worktree)) {
      run("git", ["worktree", "remove", worktree], root);
      rmSync(temp, { recursive: true, force: true });
    } else if (added) {
      console.error(`Preparation stopped; inspect the preserved worktree: ${worktree}`);
    } else {
      rmSync(temp, { recursive: true, force: true });
    }
  }
}

function tagMergedRelease(repo: string, pr: PullRequest): string {
  requireReleasePr(repo, pr);
  ensureReleaseProtection(repo);
  if (!pr.merged || !pr.merge_commit_sha) throw new Error("Release PR has not merged.");
  const version = releaseVersion(pr.head.ref);
  for (const workflow of ["ci.yml", "release-pr.yml"]) {
    if (!runPassed(latestRun(repo, workflow, pr.head.sha, "pull_request"))) {
      throw new Error(`No successful ${workflow} run for the merged release PR head.`);
    }
  }
  run("git", ["fetch", "origin", "main", pr.head.sha, pr.merge_commit_sha], root);
  const tree = (sha: string) => run("git", ["rev-parse", `${sha}^{tree}`], root);
  if (tree(pr.head.sha) !== tree(pr.merge_commit_sha)) {
    throw new Error("Merged tree differs from the validated release PR; refusing to tag.");
  }
  run("git", ["merge-base", "--is-ancestor", pr.merge_commit_sha, "origin/main"], root);
  const pkg = JSON.parse(
    run("git", ["show", `${pr.merge_commit_sha}:crates/ox_content_napi/package.json`], root),
  );
  if (pkg.version !== version)
    throw new Error("Release branch and merged package version disagree.");
  const tag = `v${version}`;
  const refs = run(
    "git",
    ["ls-remote", "--tags", "origin", `refs/tags/${tag}`, `refs/tags/${tag}^{}`],
    root,
  );
  if (refs) {
    const lines = refs.split("\n");
    const commit = (lines.find((line) => line.endsWith("^{}")) ?? lines[0]).split(/\s+/)[0];
    if (commit !== pr.merge_commit_sha) throw new Error(`${tag} points to a different commit.`);
  } else {
    // A user-authenticated push event starts both publishers (GITHUB_TOKEN would not).
    api(repo, "git/refs", { ref: `refs/tags/${tag}`, sha: pr.merge_commit_sha });
  }
  return tag;
}

async function main(): Promise<void> {
  const args = process.argv.slice(2).filter((arg) => arg !== "--");
  if (args.includes("--prepare-only")) {
    const input = args.find((arg) => arg !== "--prepare-only") ?? "patch";
    if (args.length > 2) throw new Error("Unexpected release arguments.");
    prepareRelease(input);
    return;
  }
  if (args[0] === "--help") {
    console.log(
      "vp run release [patch|minor|major|alpha|beta|x.y.z]\nvp run release --resume <pr-number>",
    );
    return;
  }
  const resume = args[0] === "--resume";
  if (
    (resume && (args.length !== 2 || !/^[1-9]\d*$/.test(args[1]))) ||
    (!resume && args.length > 1)
  ) {
    throw new Error("Usage: vp run release [version] | --resume <pr-number>");
  }
  const repo = run(
    "gh",
    ["repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"],
    root,
  );
  const login = run("gh", ["api", "user", "--jq", ".login"], root);
  authorPermission(repo, login);
  ensureReleaseProtection(repo, true);
  const number = resume ? Number(args[1]) : createReleasePr(repo, args[0] ?? "patch");
  try {
    console.log(`Release PR #${number}; resume with: vp run release --resume ${number}`);
    const pr = await mergeRelease(repo, number);
    const tag = tagMergedRelease(repo, pr);
    await watchPublication(repo, pr.merge_commit_sha!, tag, resume);
    console.log(`Released ${tag}: https://github.com/${repo}/releases/tag/${tag}`);
  } catch (error) {
    console.error(`Resume with: vp run release --resume ${number}`);
    throw error;
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
});
