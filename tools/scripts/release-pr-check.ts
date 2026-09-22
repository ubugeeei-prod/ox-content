import { spawnSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import { api, isCurrent, requireReleasePr, run, type PullRequest } from "./release-github.ts";
import { releaseVersion, requireSuccessfulJobs } from "./release-policy.ts";
import { NPM_PACKAGES } from "./release-targets.ts";

const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH!, "utf8"));
const repo = process.env.GITHUB_REPOSITORY!;
const pr = api<PullRequest>(repo, `pulls/${event.pull_request.number}`);
if (pr.head.sha !== event.pull_request.head.sha)
  throw new Error("PR head changed; wait for its new run.");

function versionAt(ref: string, file: string): string | undefined {
  if (spawnSync("git", ["cat-file", "-e", `${ref}:${file}`], { stdio: "ignore" }).status !== 0)
    return undefined;
  const text = run("git", ["show", `${ref}:${file}`]);
  return file.endsWith(".json")
    ? JSON.parse(text).version
    : text.match(/^version = "([^"]+)"/m)?.[1];
}

const metadata = [
  "Cargo.toml",
  "editors/zed/extension.toml",
  ...NPM_PACKAGES.map((dir) => `${dir}/package.json`),
];
// Inspect version fields, not labels: removing a label or renaming a PR cannot bypass validation.
const versionChanged = metadata.some(
  (file) =>
    versionAt(event.pull_request.base.sha, file) !== undefined &&
    versionAt(pr.head.sha, file) !== versionAt(event.pull_request.base.sha, file),
);
const release = versionChanged || pr.head.ref.startsWith("release/");
const requested = event.pull_request.labels.some(
  (label: { name: string }) => label.name === "release-validation",
);
const full = release || requested;

if (full) {
  requireReleasePr(repo, pr);
  if (!isCurrent(repo, pr))
    throw new Error("Update the release PR with the latest main, then re-run validation.");
}
if (release) {
  const version = releaseVersion(pr.head.ref);
  if (!versionChanged) throw new Error("Release PR must change the version.");
  for (const file of metadata) {
    if (versionAt(pr.head.sha, file) !== version)
      throw new Error(`Release version mismatch: ${file}`);
  }
}

if (process.argv.includes("--gate")) {
  const results = JSON.parse(process.env.RELEASE_JOB_RESULTS!);
  requireSuccessfulJobs({ detect: results.detect.result });
  if (full) {
    requireSuccessfulJobs(
      Object.fromEntries(
        Object.entries(results).map(([key, value]) => [key, (value as { result: string }).result]),
      ),
    );
  }
  console.log(
    full
      ? "Release validation passed on current main."
      : "Ordinary PR; release validation is not required.",
  );
} else {
  appendFileSync(process.env.GITHUB_OUTPUT!, `full=${full}\n`);
}
