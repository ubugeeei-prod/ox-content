#!/usr/bin/env node
import { parseArgs, usage } from "./issue-agent/args.mjs";
import { getIssue, listOpenIssues, resolveContext } from "./issue-agent/github.mjs";
import { processIssue } from "./issue-agent/process.mjs";
import { assertAvailable, requiredNumber } from "./issue-agent/utils.mjs";
import { watchIssues } from "./issue-agent/watch.mjs";

const command = process.argv[2] ?? "help";
const args = parseArgs(process.argv.slice(3));

async function main() {
  if (args.help || command === "help" || command === "--help" || command === "-h") {
    process.stdout.write(usage);
    return;
  }

  const context = resolveContext(args);

  assertAvailable("gh", ["--version"]);
  assertAvailable("git", ["--version"]);

  if (command === "list") {
    for (const issue of listOpenIssues(context.repo)) {
      process.stdout.write(`#${issue.number} ${issue.title}\n`);
    }
    return;
  }

  assertAvailable("codex", ["--version"]);

  if (command === "run") {
    const issueNumber = requiredNumber(args.issue, "--issue");
    const issue = getIssue(context.repo, issueNumber);
    await processIssue({ ...context, args, issue });
    return;
  }

  if (command === "watch") {
    await watchIssues({ ...context, args });
    return;
  }

  throw new Error(`Unknown command: ${command}\n\n${usage}`);
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
