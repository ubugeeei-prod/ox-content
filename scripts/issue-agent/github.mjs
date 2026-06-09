import { join } from "node:path";
import { run } from "./utils.mjs";

export function resolveContext(args) {
  const root = run("git", ["rev-parse", "--show-toplevel"]).trim();
  const repo =
    args.repo ??
    run("gh", ["repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"], {
      cwd: root,
    }).trim();
  const base =
    args.base ??
    run(
      "gh",
      ["repo", "view", repo, "--json", "defaultBranchRef", "--jq", ".defaultBranchRef.name"],
      { cwd: root },
    ).trim();
  const stateFile = args.stateFile ?? join(root, ".cache", "issue-agent", "state.json");

  return { base, repo, root, stateFile };
}

export function listOpenIssues(repo) {
  return JSON.parse(
    run("gh", [
      "issue",
      "list",
      "--repo",
      repo,
      "--state",
      "open",
      "--limit",
      "100",
      "--json",
      "number,title,body,author,labels,assignees,state,url,createdAt",
    ]),
  );
}

export function getIssue(repo, issueNumber) {
  return JSON.parse(
    run("gh", [
      "issue",
      "view",
      String(issueNumber),
      "--repo",
      repo,
      "--json",
      "number,title,body,author,labels,assignees,state,url",
    ]),
  );
}
