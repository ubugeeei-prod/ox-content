export const RELEASE_CHECK = "Release gate";
export const RELEASE_RULESET = "Release pull requests";
export const VERSION_PATTERN =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[a-zA-Z0-9]+(?:\.[a-zA-Z0-9]+)*)?$/;

export type Permission = {
  role_name?: string;
  user?: { permissions?: { maintain?: boolean; admin?: boolean } };
};

export function requireMaintainer(permission: Permission): void {
  if (
    !["maintain", "admin"].includes(permission.role_name ?? "") &&
    !permission.user?.permissions?.maintain &&
    !permission.user?.permissions?.admin
  ) {
    throw new Error("Release PR authors must have Maintain or Admin repository permission.");
  }
}

/** A maintenance line cut from a release tag, such as `v3.2.x`. */
const MAINTENANCE_BRANCH = /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.x$/;

export function isReleaseBase(ref: string): boolean {
  return ref === "main" || MAINTENANCE_BRANCH.test(ref);
}

export function requireReleaseBase(ref: string): void {
  if (!isReleaseBase(ref)) {
    throw new Error(`Releases target main or a vX.Y.x maintenance branch, not ${ref}.`);
  }
}

/** A maintenance line only ships versions of its own major.minor. */
export function requireVersionOnBase(version: string, base: string): void {
  requireReleaseBase(base);
  const line = base.match(MAINTENANCE_BRANCH);
  if (line && !version.startsWith(`${line[1]}.${line[2]}.`)) {
    throw new Error(`v${version} does not belong to the ${base} maintenance line.`);
  }
}

export function releaseVersion(branch: string): string {
  const version = branch.replace(/^release\/v/, "");
  if (!branch.startsWith("release/v") || !VERSION_PATTERN.test(version)) {
    throw new Error(`Invalid release branch: ${branch}`);
  }
  return version;
}

export function requireSuccessfulJobs(results: Record<string, string>): void {
  for (const [job, result] of Object.entries(results)) {
    if (result !== "success") throw new Error(`${job} did not pass (${result}).`);
  }
}

export type Ruleset = {
  name: string;
  enforcement: string;
  // GitHub omits bypass_actors unless the token can edit the ruleset.
  bypass_actors?: unknown[];
  current_user_can_bypass?: string;
  conditions: { ref_name: { include: string[]; exclude: string[] } };
  rules: {
    type: string;
    parameters?: {
      strict_required_status_checks_policy?: boolean;
      required_status_checks?: { context: string; integration_id?: number }[];
    };
  }[];
};

export function hasReleaseProtection(rule: Ruleset, base = "main"): boolean {
  return (
    rule.name === RELEASE_RULESET &&
    rule.enforcement === "active" &&
    (rule.bypass_actors === undefined || rule.bypass_actors.length === 0) &&
    (!rule.current_user_can_bypass || rule.current_user_can_bypass === "never") &&
    rule.conditions.ref_name.include.includes("refs/heads/main") &&
    rule.conditions.ref_name.include.includes(`refs/heads/${base}`) &&
    rule.conditions.ref_name.exclude.length === 0 &&
    rule.rules.some((item) => item.type === "pull_request") &&
    rule.rules.some(
      (item) =>
        item.type === "required_status_checks" &&
        item.parameters?.strict_required_status_checks_policy === true &&
        item.parameters.required_status_checks?.some(
          (check) => check.context === RELEASE_CHECK && check.integration_id === 15368,
        ),
    )
  );
}
