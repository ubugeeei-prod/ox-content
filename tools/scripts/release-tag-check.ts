import {
  api,
  ensureReleaseProtection,
  latestRun,
  requireReleasePr,
  run,
  runPassed,
  type PullRequest,
} from "./release-github.ts";
import { releaseVersion } from "./release-policy.ts";

const repo = process.env.GITHUB_REPOSITORY!;
const sha = process.env.GITHUB_SHA!;
const tag = process.env.GITHUB_REF_NAME!;
const prs = api<PullRequest[]>(repo, `commits/${sha}/pulls?per_page=100`);
const candidate = prs.find((item) => item.merge_commit_sha === sha && item.base.ref === "main");
if (!candidate) throw new Error("Release tags must point to a merged release PR.");
const pr = api<PullRequest>(repo, `pulls/${candidate.number}`);
if (!pr.merged) throw new Error("Release PR has not merged.");
requireReleasePr(repo, pr);
ensureReleaseProtection(repo);
if (tag !== `v${releaseVersion(pr.head.ref)}`)
  throw new Error("Tag and release PR version differ.");
for (const workflow of ["ci.yml", "release-pr.yml"]) {
  if (!runPassed(latestRun(repo, workflow, pr.head.sha, "pull_request"))) {
    throw new Error(`Missing successful ${workflow} validation for release PR #${pr.number}.`);
  }
}
run("git", ["fetch", "origin", "main", pr.head.sha]);
run("git", ["merge-base", "--is-ancestor", sha, "origin/main"]);
if (
  run("git", ["rev-parse", `${sha}^{tree}`]) !== run("git", ["rev-parse", `${pr.head.sha}^{tree}`])
) {
  throw new Error("The release tag differs from the validated PR tree.");
}
console.log(`Authorized ${tag} from ${pr.html_url}`);
