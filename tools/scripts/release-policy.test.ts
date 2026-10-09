import { describe, expect, it } from "vite-plus/test";
import {
  hasReleaseProtection,
  isReleaseBase,
  releaseVersion,
  requireMaintainer,
  requireSuccessfulJobs,
  requireVersionOnBase,
  type Ruleset,
} from "./release-policy.ts";

const protectedRule: Ruleset = {
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

describe("release authorization", () => {
  it.each(["maintain", "admin"])("accepts %s", (role_name) => {
    expect(() => requireMaintainer({ role_name })).not.toThrow();
  });
  it.each(["write", "read", "triage", "none", "custom-write"])("rejects %s", (role_name) => {
    expect(() => requireMaintainer({ role_name })).toThrow(/Maintain or Admin/);
  });
  it("accepts custom roles only with base maintain permissions", () => {
    expect(() =>
      requireMaintainer({ role_name: "custom", user: { permissions: { maintain: true } } }),
    ).not.toThrow();
  });
  it.each(["failure", "skipped", "cancelled", "timed_out"])(
    "fails closed for a %s validation job",
    (status) => {
      expect(() => requireSuccessfulJobs({ napi: "success", packages: status })).toThrow(
        /packages/,
      );
    },
  );
});

describe("release branch and protection", () => {
  it.each(["release/v1.2.3", "release/v1.2.3-alpha.0"])("parses %s", (branch) => {
    expect(releaseVersion(branch)).toBe(branch.slice(9));
  });
  it.each(["main", "release/v1.2", "release/v01.2.3", "release/v1.2.3;echo", "release/v1.2.3/a"])(
    "rejects %s",
    (branch) => {
      expect(() => releaseVersion(branch)).toThrow();
    },
  );
  it("requires a strict, non-bypassable Actions check on main", () => {
    expect(hasReleaseProtection(protectedRule)).toBe(true);
    expect(hasReleaseProtection({ ...protectedRule, bypass_actors: undefined })).toBe(true);
    expect(
      hasReleaseProtection({
        ...protectedRule,
        bypass_actors: undefined,
        current_user_can_bypass: "always",
      }),
    ).toBe(false);
    expect(hasReleaseProtection({ ...protectedRule, bypass_actors: [{ actor_id: 5 }] })).toBe(
      false,
    );
    expect(hasReleaseProtection({ ...protectedRule, enforcement: "disabled" })).toBe(false);
    expect(hasReleaseProtection({ ...protectedRule, rules: protectedRule.rules.slice(1) })).toBe(
      false,
    );
    const loose = structuredClone(protectedRule);
    loose.rules[1].parameters!.strict_required_status_checks_policy = false;
    expect(hasReleaseProtection(loose)).toBe(false);
    const wrongApp = structuredClone(protectedRule);
    wrongApp.rules[1].parameters!.required_status_checks![0].integration_id = 1;
    expect(hasReleaseProtection(wrongApp)).toBe(false);
  });
});

describe("maintenance lines", () => {
  it.each(["main", "v3.2.x", "v10.0.x", "v0.1.x"])("accepts %s as a release base", (ref) => {
    expect(isReleaseBase(ref)).toBe(true);
  });
  it.each(["develop", "v3.x", "v3.2.1", "v03.2.x", "release/v3.2.x", "v3.2.x;echo", "3.2.x"])(
    "rejects %s as a release base",
    (ref) => {
      expect(isReleaseBase(ref)).toBe(false);
      expect(() => requireVersionOnBase("3.2.14", ref)).toThrow(/maintenance branch/);
    },
  );
  it.each(["3.2.14", "3.2.15-beta.0"])("ships %s from v3.2.x", (version) => {
    expect(() => requireVersionOnBase(version, "v3.2.x")).not.toThrow();
  });
  it.each(["3.3.0", "3.20.1", "4.2.0", "13.2.0"])("keeps %s off v3.2.x", (version) => {
    expect(() => requireVersionOnBase(version, "v3.2.x")).toThrow(/v3\.2\.x maintenance line/);
  });
  it("lets main ship any version", () => {
    expect(() => requireVersionOnBase("3.2.14", "main")).not.toThrow();
  });
  it("requires the ruleset to cover both main and the maintenance line", () => {
    expect(hasReleaseProtection(protectedRule, "v3.2.x")).toBe(false);
    const both = structuredClone(protectedRule);
    both.conditions.ref_name.include.push("refs/heads/v3.2.x");
    expect(hasReleaseProtection(both, "v3.2.x")).toBe(true);
    const lineOnly = structuredClone(protectedRule);
    lineOnly.conditions.ref_name.include = ["refs/heads/v3.2.x"];
    expect(hasReleaseProtection(lineOnly, "v3.2.x")).toBe(false);
  });
});
