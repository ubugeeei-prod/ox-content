import { execFileSync } from "node:child_process";
import { setTimeout as sleep } from "node:timers/promises";
import {
  hasReleaseProtection,
  RELEASE_CHECK,
  RELEASE_RULESET,
  requireMaintainer,
  releaseVersion,
  type Permission,
  type Ruleset,
} from "./release-policy.ts";

export function run(command: string, args: string[], cwd = process.cwd()): string {
  return execFileSync(command, args, {
    cwd,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }).trim();
}

export function api<T>(repo: string, route: string, body?: unknown, method = "POST"): T {
  const args = ["api", `repos/${repo}/${route}`];
  if (body !== undefined) args.push("--method", method, "--input", "-");
  return JSON.parse(
    execFileSync("gh", args, {
      encoding: "utf8",
      input: body === undefined ? undefined : JSON.stringify(body),
      stdio: ["pipe", "pipe", "inherit"],
    }),
  );
}

export type PullRequest = {
  number: number;
  html_url: string;
  state: string;
  draft: boolean;
  merged: boolean;
  merge_commit_sha: string | null;
  user: { login: string };
  head: { sha: string; ref: string; repo: { full_name: string } | null };
  base: { ref: string; sha: string };
};

export function authorPermission(repo: string, login: string): void {
  requireMaintainer(api<Permission>(repo, `collaborators/${encodeURIComponent(login)}/permission`));
}

export function requireReleasePr(repo: string, pr: PullRequest): void {
  if (pr.base.ref !== "main" || pr.head.repo?.full_name !== repo || pr.draft) {
    throw new Error("Release requires a non-draft, same-repository PR targeting main.");
  }
  authorPermission(repo, pr.user.login);
}

export function isCurrent(repo: string, pr: PullRequest): boolean {
  const comparison = api<{ behind_by: number }>(repo, `compare/main...${pr.head.sha}`);
  return comparison.behind_by === 0;
}

export function ensureReleaseProtection(repo: string, install = false): void {
  const summaries = api<{ id: number; name: string }[]>(repo, "rulesets?per_page=100");
  const summary = summaries.find((item) => item.name === RELEASE_RULESET);
  if (summary && hasReleaseProtection(api<Ruleset>(repo, `rulesets/${summary.id}`))) return;
  if (!install || summary) {
    throw new Error(
      `An active, non-bypassable ${RELEASE_RULESET} ruleset with strict ${RELEASE_CHECK} is required. See docs/releasing.md.`,
    );
  }
  api(repo, "contents/.github/workflows/release-pr.yml?ref=main");
  // One-time repository setup. Maintain users can release once an Admin installs this rule.
  api(repo, "rulesets", {
    name: RELEASE_RULESET,
    target: "branch",
    enforcement: "active",
    bypass_actors: [],
    conditions: { ref_name: { include: ["refs/heads/main"], exclude: [] } },
    rules: [
      {
        type: "pull_request",
        parameters: {
          required_approving_review_count: 0,
          dismiss_stale_reviews_on_push: false,
          require_code_owner_review: false,
          require_last_push_approval: false,
          required_review_thread_resolution: false,
          allowed_merge_methods: ["squash", "merge"],
        },
      },
      {
        type: "required_status_checks",
        parameters: {
          strict_required_status_checks_policy: true,
          do_not_enforce_on_create: false,
          required_status_checks: [{ context: RELEASE_CHECK, integration_id: 15368 }],
        },
      },
    ],
  });
  ensureReleaseProtection(repo);
}

export type WorkflowRun = {
  id: number;
  head_sha: string;
  head_branch: string;
  status: string;
  conclusion: string | null;
  html_url: string;
};

export function latestRun(
  repo: string,
  workflow: string,
  sha: string,
  event: string,
  branch?: string,
): WorkflowRun | undefined {
  const runs = api<{ workflow_runs: WorkflowRun[] }>(
    repo,
    `actions/workflows/${workflow}/runs?event=${event}&head_sha=${sha}&per_page=100`,
  ).workflow_runs;
  return runs
    .filter((item) => !branch || item.head_branch === branch)
    .sort((a, b) => b.id - a.id)[0];
}

export function runPassed(run: WorkflowRun | undefined): boolean {
  if (!run || run.status !== "completed") return false;
  if (run.conclusion !== "success") throw new Error(`Workflow ${run.conclusion}: ${run.html_url}`);
  return true;
}

export async function mergeRelease(repo: string, number: number): Promise<PullRequest> {
  const deadline = Date.now() + 4 * 60 * 60 * 1000;
  while (Date.now() < deadline) {
    const pr = api<PullRequest>(repo, `pulls/${number}`);
    requireReleasePr(repo, pr);
    releaseVersion(pr.head.ref);
    if (pr.merged) return pr;
    if (pr.state !== "open") throw new Error(`PR is closed: ${pr.html_url}`);
    if (!isCurrent(repo, pr)) {
      console.log("main advanced; updating the release PR and waiting for fresh validation.");
      api(repo, `pulls/${number}/update-branch`, { expected_head_sha: pr.head.sha }, "PUT");
      await sleep(15_000);
      continue;
    }
    const workflows = ["ci.yml", "release-pr.yml"];
    const passed = workflows.map((workflow) =>
      runPassed(latestRun(repo, workflow, pr.head.sha, "pull_request")),
    );
    if (passed.every(Boolean)) {
      ensureReleaseProtection(repo);
      // GitHub's strict rule closes the race if main moves after isCurrent().
      try {
        const result = api<{ merged: boolean }>(
          repo,
          `pulls/${number}/merge`,
          {
            sha: pr.head.sha,
            merge_method: "squash",
          },
          "PUT",
        );
        if (result.merged) return api<PullRequest>(repo, `pulls/${number}`);
      } catch (error) {
        if (isCurrent(repo, api<PullRequest>(repo, `pulls/${number}`))) throw error;
      }
    }
    console.log(`Waiting for release validation: ${pr.html_url}`);
    await sleep(30_000);
  }
  throw new Error(`Timed out waiting for PR #${number}; resume after checking Actions.`);
}

export async function watchPublication(
  repo: string,
  sha: string,
  tag: string,
  retry: boolean,
): Promise<void> {
  const deadline = Date.now() + 4 * 60 * 60 * 1000;
  const retried = new Set<number>();
  while (Date.now() < deadline) {
    const passed = ["publish.yml", "publish-editors.yml"].map((workflow) => {
      const item = latestRun(repo, workflow, sha, "push", tag);
      if (
        retry &&
        item?.status === "completed" &&
        item.conclusion !== "success" &&
        !retried.has(item.id)
      ) {
        run("gh", ["run", "rerun", String(item.id), "--failed", "--repo", repo]);
        retried.add(item.id);
        return false;
      }
      return runPassed(item);
    });
    if (passed.every(Boolean)) {
      run("gh", ["release", "view", tag, "--repo", repo]);
      return;
    }
    console.log(`Waiting for publication of ${tag}...`);
    await sleep(30_000);
  }
  throw new Error(`Timed out waiting for publication of ${tag}.`);
}
