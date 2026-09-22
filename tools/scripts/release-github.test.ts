import { beforeEach, describe, expect, it, vi } from "vite-plus/test";
import { execFileSync } from "node:child_process";
import { mergeRelease, runPassed, watchPublication, type PullRequest } from "./release-github.ts";

vi.mock("node:child_process", () => ({ execFileSync: vi.fn() }));
vi.mock("node:timers/promises", () => ({ setTimeout: vi.fn().mockResolvedValue(undefined) }));

const pr: PullRequest = {
  number: 123,
  html_url: "https://example.test/pull/123",
  state: "open",
  draft: false,
  merged: false,
  merge_commit_sha: null,
  user: { login: "maintainer" },
  head: { ref: "release/v1.2.3", sha: "head1", repo: { full_name: "owner/repo" } },
  base: { ref: "main", sha: "base1" },
};
const success = {
  id: 100,
  head_sha: "head1",
  head_branch: "release/v1.2.3",
  status: "completed",
  conclusion: "success",
  html_url: "https://example.test/actions/100",
};
const rule = {
  name: "Release pull requests",
  enforcement: "active",
  bypass_actors: [],
  conditions: { ref_name: { include: ["refs/heads/main"], exclude: [] } },
  rules: [
    { type: "pull_request" },
    {
      type: "required_status_checks",
      parameters: {
        strict_required_status_checks_policy: true,
        required_status_checks: [{ context: "Release gate", integration_id: 15368 }],
      },
    },
  ],
};

function mockApi(overrides: (route: string, body: unknown) => unknown = () => undefined) {
  let merged = false;
  vi.mocked(execFileSync).mockImplementation((command, args, options) => {
    expect(command).toBe("gh");
    const route = String(args![1]).replace("repos/owner/repo/", "");
    const body =
      options && "input" in options && options.input
        ? JSON.parse(String(options.input))
        : undefined;
    let response = overrides(route, body);
    if (response === undefined) {
      if (route === "pulls/123")
        response = { ...pr, merged, merge_commit_sha: merged ? "merged1" : null };
      else if (route.includes("/permission")) response = { role_name: "maintain" };
      else if (route.startsWith("compare/")) response = { behind_by: 0 };
      else if (route.includes("/runs?")) response = { workflow_runs: [success] };
      else if (route === "rulesets?per_page=100") response = [{ id: 1, name: rule.name }];
      else if (route === "rulesets/1") response = rule;
      else if (route === "pulls/123/merge") {
        merged = true;
        response = { merged: true };
      } else throw new Error(`Unexpected route ${route}`);
    }
    return JSON.stringify(response);
  });
}

beforeEach(() => vi.clearAllMocks());

describe("release orchestration", () => {
  it("waits for both CI and full validation before merging the exact head", async () => {
    let pending = true;
    mockApi((route) => {
      if (route.includes("ci.yml/runs?") && pending) {
        pending = false;
        return { workflow_runs: [] };
      }
    });
    expect((await mergeRelease("owner/repo", 123)).merged).toBe(true);
    const call = vi
      .mocked(execFileSync)
      .mock.calls.find(([, args]) => args?.[1] === "repos/owner/repo/pulls/123/merge");
    expect(JSON.parse(String(call?.[2]?.input))).toEqual({ sha: "head1", merge_method: "squash" });
  });
  it("updates stale main before considering green checks", async () => {
    let stale = true;
    mockApi((route, body) => {
      if (route.startsWith("compare/")) return { behind_by: stale ? 1 : 0 };
      if (route.endsWith("/update-branch")) {
        expect(body).toEqual({ expected_head_sha: "head1" });
        stale = false;
        return {};
      }
    });
    await mergeRelease("owner/repo", 123);
    expect(stale).toBe(false);
  });
  it("refuses to resume an ordinary PR even with successful checks", async () => {
    mockApi((route) =>
      route === "pulls/123" ? { ...pr, head: { ...pr.head, ref: "fix/ordinary" } } : undefined,
    );
    await expect(mergeRelease("owner/repo", 123)).rejects.toThrow(/Invalid release branch/);
  });
  it("revalidates after main advances at merge time", async () => {
    let race = false;
    let retried = false;
    mockApi((route) => {
      if (route.endsWith("/merge") && !retried) {
        race = true;
        throw new Error("Base branch changed");
      }
      if (route.startsWith("compare/")) return { behind_by: race ? 1 : 0 };
      if (route.endsWith("/update-branch")) {
        race = false;
        retried = true;
        return {};
      }
    });
    expect((await mergeRelease("owner/repo", 123)).merged).toBe(true);
    expect(retried).toBe(true);
  });
  it("stops before merge when validation fails", async () => {
    mockApi((route) =>
      route.includes("release-pr.yml/runs?")
        ? { workflow_runs: [{ ...success, conclusion: "failure" }] }
        : undefined,
    );
    await expect(mergeRelease("owner/repo", 123)).rejects.toThrow(/Workflow failure/);
    expect(
      vi.mocked(execFileSync).mock.calls.some(([, args]) => String(args?.[1]).endsWith("/merge")),
    ).toBe(false);
  });
  it("rechecks author permissions even for a merged PR on resume", async () => {
    mockApi((route) =>
      route.includes("/permission")
        ? { role_name: "write" }
        : route === "pulls/123"
          ? { ...pr, merged: true }
          : undefined,
    );
    await expect(mergeRelease("owner/repo", 123)).rejects.toThrow(/Maintain or Admin/);
  });
  it("rejects disabled strict protection even when all checks are green", async () => {
    mockApi((route) => (route === "rulesets/1" ? { ...rule, enforcement: "disabled" } : undefined));
    await expect(mergeRelease("owner/repo", 123)).rejects.toThrow(/ruleset/);
  });
  it("does not turn a skipped workflow into successful validation", () => {
    expect(() => runPassed({ ...success, conclusion: "skipped" })).toThrow();
    expect(runPassed(undefined)).toBe(false);
  });
  it("reports publication failure instead of claiming a release", async () => {
    mockApi((route) =>
      route.includes("/runs?")
        ? { workflow_runs: [{ ...success, head_branch: "v1.2.3", conclusion: "failure" }] }
        : undefined,
    );
    await expect(watchPublication("owner/repo", "head1", "v1.2.3", false)).rejects.toThrow(
      /Workflow failure/,
    );
  });
});
