import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { getIssue } from "./github.mjs";
import { buildPrTitle } from "./title.mjs";
import { ensureClean, names, run, slug } from "./utils.mjs";

const restrictedPaths = [
  ".github/workflows/",
  ".github/actions/",
  ".github/codex/",
  "scripts/issue-agent.mjs",
  "scripts/issue-agent/",
];

export async function processIssue({ args, issue, repo, base, root }) {
  ensureClean(root);

  const issueDetails = getIssue(repo, issue.number);
  if (issueDetails.state !== "OPEN") {
    process.stdout.write(`Issue #${issue.number} is not open. Skipping.\n`);
    return { branch: null, prUrl: null, status: "skipped" };
  }

  const branch = `codex/issue-${issueDetails.number}-${slug(issueDetails.title)}-${Date.now()}`;
  const prTitle = buildPrTitle(issueDetails);
  const prompt = renderPrompt(root, issueDetails);
  const outputFile = join(
    root,
    ".cache",
    "issue-agent",
    `codex-output-${issueDetails.number}-${Date.now()}.md`,
  );

  mkdirSync(join(root, ".cache", "issue-agent"), { recursive: true });

  run("git", ["fetch", "origin", base, "--no-tags"], { cwd: root, stdio: "inherit" });
  run("git", ["switch", "-c", branch, `origin/${base}`], { cwd: root, stdio: "inherit" });

  process.stdout.write(`Running Codex for issue #${issueDetails.number} on ${branch}\n`);
  run(
    "codex",
    ["exec", "--sandbox", "workspace-write", "--ephemeral", "--cd", root, "-o", outputFile, "-"],
    {
      cwd: root,
      input: prompt,
      stdio: ["pipe", "inherit", "inherit"],
    },
  );

  run("git", ["add", "-N", "."], { cwd: root });
  const changedFiles = run("git", ["diff", "--name-only", "HEAD"], { cwd: root })
    .trim()
    .split("\n")
    .filter(Boolean);

  if (changedFiles.length === 0) {
    process.stdout.write(`Codex produced no patch for issue #${issueDetails.number}.\n`);
    return { branch, prUrl: null, status: "no-patch" };
  }

  const restrictedChanges = changedFiles.filter((file) =>
    restrictedPaths.some((path) => file === path || file.startsWith(path)),
  );
  if (restrictedChanges.length > 0) {
    process.stderr.write(
      `Refusing to open a PR because restricted files changed:\n${restrictedChanges.join("\n")}\n`,
    );
    return { branch, prUrl: null, status: "restricted" };
  }

  run("git", ["add", "-A"], { cwd: root, stdio: "inherit" });
  run("git", ["commit", "-m", prTitle], { cwd: root, stdio: "inherit" });
  run("git", ["push", "-u", "origin", branch], { cwd: root, stdio: "inherit" });

  const bodyFile = writePrBody(issueDetails, prTitle, outputFile);
  const prUrl = run(
    "gh",
    [
      "pr",
      "create",
      "--repo",
      repo,
      "--base",
      base,
      "--head",
      branch,
      "--title",
      prTitle,
      "--body-file",
      bodyFile,
    ],
    { cwd: root },
  ).trim();

  run(
    "gh",
    [
      "issue",
      "comment",
      String(issueDetails.number),
      "--repo",
      repo,
      "--body",
      `Created pull request: ${prUrl}`,
    ],
    { cwd: root },
  );

  if (!args.noWatchCi) {
    run("gh", ["pr", "checks", prUrl, "--repo", repo, "--watch", "--interval", "30"], {
      cwd: root,
      stdio: "inherit",
    });
  }

  return { branch, prUrl, status: "pr-created" };
}

function renderPrompt(root, issue) {
  const template = readFileSync(
    join(root, ".github", "codex", "prompts", "implement-issue.md"),
    "utf8",
  );
  const replacements = {
    ISSUE_ASSIGNEES: names(issue.assignees),
    ISSUE_AUTHOR: issue.author?.login ?? "(unknown)",
    ISSUE_BODY: issue.body || "(empty)",
    ISSUE_LABELS: names(issue.labels),
    ISSUE_NUMBER: String(issue.number),
    ISSUE_TITLE: issue.title,
    ISSUE_URL: issue.url,
  };

  return template.replace(/\{\{([A-Z_]+)\}\}/g, (_, key) => replacements[key] ?? "");
}

function writePrBody(issue, prTitle, outputFile) {
  const bodyFile = join(mkdtempSync(join(tmpdir(), "issue-agent-")), "pr-body.md");
  const codexOutput = existsSync(outputFile) ? readFileSync(outputFile, "utf8").trim() : "";

  writeFileSync(
    bodyFile,
    `## Summary

- Implements #${issue.number} with the local issue agent.
- Generated title: \`${prTitle}\`

## Risk

- Risk level: medium
- User or production impact: Generated from issue context and requires maintainer review.
- Rollback plan: Revert this PR.

## Validation

- [ ] Automated tests or checks run: Local issue agent ran Codex before opening this PR.
- [ ] Manual verification completed: Maintainer review required.
- [ ] Edge cases or failure modes considered: Review the issue and generated diff before merge.

## Release and docs checklist

- [ ] Documentation, comments, or runbooks updated, or not needed.
- [ ] Configuration, migration, or environment changes documented, or not needed.
- [ ] Observability, logging, and alerting impact considered, or not needed.
- [ ] Security, privacy, and dependency impact considered, or not needed.

Closes #${issue.number}

Source issue: ${issue.url}

${codexOutput ? `## Codex final message\n\n${codexOutput}\n` : ""}`,
  );

  return bodyFile;
}
