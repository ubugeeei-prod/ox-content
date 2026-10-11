import { spawnSync } from "node:child_process";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { NPM_PACKAGES } from "./release-targets.ts";

const script = resolve("tools/scripts/release-package-check.ts");
const roots: string[] = [];
afterEach(() => roots.splice(0).forEach((root) => rmSync(root, { recursive: true, force: true })));

describe("release npm registration checks", () => {
  it("uses the same public-registration check in publishing preflight", () => {
    const fixture = createFixture();
    const result = runCheck(fixture, "public", true);
    expect(result.status).toBe(0);
    expect(result.stdout).toContain("Every package is registered as public on npm");
    expect(readFileSync(fixture.log, "utf8")).toContain('"access","get","status"');
  });
  it("accepts public registration before the first version is published", () => {
    const fixture = createFixture();
    const result = runCheck(fixture, "public");
    expect(result.status).toBe(0);
    const calls = readFileSync(fixture.log, "utf8");
    expect(calls).toContain('"access","get","status"');
    expect(calls).not.toContain('"view"');
    expect(calls).toContain('"pack"');
  });

  it.each(["restricted", "missing"])("rejects %s registration before packing", (visibility) => {
    const fixture = createFixture();
    const result = runCheck(fixture, visibility);
    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain("must be registered as a public npm package");
    expect(readFileSync(fixture.log, "utf8")).not.toContain('"pack"');
  });
});

function createFixture() {
  const root = mkdtempSync(join(tmpdir(), "ox-npm-release-check-"));
  roots.push(root);
  const bin = join(root, "bin");
  mkdirSync(bin);
  mkdirSync(join(root, "binding-packages"));
  const packages = [
    ...NPM_PACKAGES.filter((dir) => dir !== "npm/vscode-ox-content"),
    "crates/ox_content_wasm/pkg",
  ];
  for (const [index, dir] of packages.entries()) {
    mkdirSync(join(root, dir), { recursive: true });
    writeFileSync(
      join(root, dir, "package.json"),
      JSON.stringify({ name: `@example/package-${index}` }),
    );
  }
  const log = join(root, "calls.jsonl");
  for (const command of ["npm", "vp"]) {
    const executable = join(bin, command);
    writeFileSync(
      executable,
      `#!/usr/bin/env node
const fs = require('node:fs');
const args = process.argv.slice(2);
fs.appendFileSync(process.env.REGISTRATION_LOG, JSON.stringify(args) + '\\n');
if (args[0] === 'access') {
  console.log(JSON.stringify(process.env.REGISTRATION_VISIBILITY === 'missing' ? {} : { [args[3]]: process.env.REGISTRATION_VISIBILITY }));
}
`,
    );
    chmodSync(executable, 0o755);
  }
  return { root, bin, log };
}

function runCheck(
  fixture: ReturnType<typeof createFixture>,
  visibility: string,
  preflight = false,
) {
  const args = preflight
    ? [resolve("tools/scripts/npm-registry-check.ts"), NPM_PACKAGES[0]]
    : [script];
  return spawnSync(process.execPath, args, {
    cwd: fixture.root,
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${fixture.bin}${process.platform === "win32" ? ";" : ":"}${process.env.PATH}`,
      REGISTRATION_VISIBILITY: visibility,
      REGISTRATION_LOG: fixture.log,
    },
  });
}
