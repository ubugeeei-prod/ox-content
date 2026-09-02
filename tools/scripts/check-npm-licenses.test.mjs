import { describe, expect, it } from "vite-plus/test";
import { checkLicensePolicy, isPnpmRuntimePackage } from "./check-npm-licenses.mjs";

const blockedLicense = "MIT OR BSD OR bsd OR GPLv3 OR ISC OR zlib";
const policy = {
  licenses: {
    allowed: ["MIT"],
    exceptions: [],
  },
};

describe("check-npm-licenses", () => {
  it("ignores pnpm managed devEngines runtime packages", () => {
    const runtimePackage = {
      name: "node",
      versions: ["26.8.1"],
      paths: ["/repo/node_modules/.pnpm/node@runtime+26.8.1/node_modules/node"],
    };

    expect(isPnpmRuntimePackage(runtimePackage)).toBe(true);
    expect(checkLicensePolicy(reportFor(runtimePackage), policy).violations).toEqual([]);
  });

  it("still blocks regular packages with a runtime package name", () => {
    const dependencyPackage = {
      name: "node",
      versions: ["26.8.1"],
      paths: ["/repo/node_modules/.pnpm/node@26.8.1/node_modules/node"],
    };

    expect(isPnpmRuntimePackage(dependencyPackage)).toBe(false);
    expect(checkLicensePolicy(reportFor(dependencyPackage), policy).violations).toEqual([
      "node@26.8.1 uses MIT OR BSD OR bsd OR GPLv3 OR ISC OR zlib",
    ]);
  });
});

function reportFor(pkg) {
  return {
    [blockedLicense]: {
      packages: [pkg],
    },
  };
}
