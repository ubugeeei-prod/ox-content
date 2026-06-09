import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

export function assertAvailable(binary, versionArgs) {
  const result = spawnSync(binary, versionArgs, { encoding: "utf8" });
  if (result.status !== 0) {
    throw new Error(`Required command is not available: ${binary}`);
  }
}

export function ensureClean(root) {
  const status = run("git", ["status", "--porcelain"], { cwd: root }).trim();
  if (status) {
    throw new Error(`Working tree must be clean before running issue-agent:\n${status}`);
  }
}

export function loadState(file) {
  if (!existsSync(file)) {
    return { initialized: false, processedIssues: {}, seenIssues: [] };
  }

  return JSON.parse(readFileSync(file, "utf8"));
}

export function markSeen(state, issueNumber) {
  if (!state.seenIssues.includes(issueNumber)) {
    state.seenIssues.push(issueNumber);
  }
}

export function names(items = []) {
  const values = items.map((item) => item.name ?? item.login).filter(Boolean);
  return values.length > 0 ? values.join(", ") : "(none)";
}

export function requiredNumber(value, name) {
  const number = Number(value);
  if (!Number.isInteger(number) || number <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return number;
}

export function run(binary, commandArgs, options = {}) {
  const result = spawnSync(binary, commandArgs, {
    cwd: options.cwd,
    encoding: "utf8",
    input: options.input,
    stdio: options.stdio ?? "pipe",
  });

  if (result.status !== 0) {
    const commandText = [binary, ...commandArgs].join(" ");
    const output = [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
    throw new Error(
      output ? `${commandText}\n${output}` : `${commandText} exited with ${result.status}`,
    );
  }

  return result.stdout ?? "";
}

export function saveState(file, state) {
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, `${JSON.stringify(state, null, 2)}\n`);
}

export function sleep(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

export function slug(value) {
  return (
    value
      .normalize("NFKD")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 48)
      .replace(/-+$/g, "") || "task"
  );
}
