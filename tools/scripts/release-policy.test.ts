import { describe, expect, it } from "vite-plus/test";
import {
  hasReleaseProtection,
  releaseVersion,
  requireMaintainer,
  requireSuccessfulJobs,
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
