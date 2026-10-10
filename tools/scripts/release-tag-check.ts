import {
  api,
  ensureReleaseProtection,
  latestRun,
  requireReleasePr,
  run,
  runPassed,
  type PullRequest,
} from "./release-github.ts";
import { isReleaseBase, releaseVersion, requireVersionOnBase } from "./release-policy.ts";

const repo = process.env.GITHUB_REPOSITORY!;
const sha = process.env.GITHUB_SHA!;
const tag = process.env.GITHUB_REF_NAME!;
const prs = api<PullRequest[]>(repo, `commits/${sha}/pulls?per_page=100`);
const candidate = prs.find((item) => item.merge_commit_sha === sha && isReleaseBase(item.base.ref));
if (!candidate) throw new Error("Release tags must point to a merged release PR.");
const pr = api<PullRequest>(repo, `pulls/${candidate.number}`);
if (!pr.merged) throw new Error("Release PR has not merged.");
requireReleasePr(repo, pr);
const base = pr.base.ref;
ensureReleaseProtection(repo, false, base);
const version = releaseVersion(pr.head.ref);
requireVersionOnBase(version, base);
if (tag !== `v${version}`) throw new Error("Tag and release PR version differ.");
for (const workflow of ["ci.yml", "release-pr.yml"]) {
  if (!runPassed(latestRun(repo, workflow, pr.head.sha, "pull_request"))) {
    throw new Error(`Missing successful ${workflow} validation for release PR #${pr.number}.`);
  }
}
run("git", ["fetch", "origin", base, pr.head.sha]);
run("git", ["merge-base", "--is-ancestor", sha, `origin/${base}`]);
// The first parent is the base at the instant of merge, even if it has since advanced.
run("git", ["merge-base", "--is-ancestor", `${sha}^1`, pr.head.sha]);
if (
  run("git", ["rev-parse", `${sha}^{tree}`]) !== run("git", ["rev-parse", `${pr.head.sha}^{tree}`])
) {
  throw new Error("The release tag differs from the validated PR tree.");
}
console.log(`Authorized ${tag} from ${pr.html_url}`);
