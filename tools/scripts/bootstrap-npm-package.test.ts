import { spawnSync } from "node:child_process";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";

const script = readFileSync(resolve("tools/scripts/bootstrap-npm-package.mjs"), "utf8");
const roots: string[] = [];
afterEach(() => roots.splice(0).forEach((root) => rmSync(root, { recursive: true, force: true })));

describe("npm first-package registration", () => {
  it("stages a separate prerelease and configures OIDC without a public publish", () => {
    const fixture = createFixture("3.3.0");
    const result = runBootstrap(fixture);

    expect(result.status).toBe(0);
    const calls = readCalls(fixture);
    const stage = calls.find((call) => call.args.includes("stage"));
    expect(stage?.manifest.version).toBe("3.3.0-registration.0");
    expect(stage?.manifest.dependencies["@ox-content/vite-plugin"]).toBe("3.3.0");
    expect(stage?.args).toContain("--provenance=false");
    expect(stage?.args.slice(-2)).toEqual(["--tag", "registration"]);
    expect(calls.some((call) => call.args.includes("trust"))).toBe(true);
    expect(calls.filter((call) => call.args.includes("publish"))).toHaveLength(1);
    expect(calls.find((call) => call.args.includes("publish"))?.args).toContain("stage");
  });

  it("keeps a prerelease's release version separate from the registration candidate", () => {
    const fixture = createFixture("3.3.0-alpha.1");
    expect(runBootstrap(fixture).status).toBe(0);
    const stage = readCalls(fixture).find((call) => call.args.includes("stage"));
    expect(stage?.manifest.version).toBe("3.3.0-alpha.1.registration.0");
  });

  it("prepares a reviewable release tarball without authentication or registry changes", () => {
    const fixture = createFixture("3.3.0");
    const result = runBootstrap(fixture, "--pack-only");
    expect(result.status).toBe(0);
    expect(result.stdout).toContain("No publication or trusted-publisher changes made");
    expect(readCalls(fixture).some((call) => call.args.includes("npm@11.19.0"))).toBe(false);
  });

  it("registers an already staged package without staging another version", () => {
    const fixture = createFixture("3.3.0");
    expect(runBootstrap(fixture, "--register-only").status).toBe(0);
    const calls = readCalls(fixture);
    expect(calls).toHaveLength(2);
    expect(calls[0].args).toContain("whoami");
    expect(calls[1].args).toContain("trust");
  });
});

function createFixture(version: string) {
  const root = mkdtempSync(join(tmpdir(), "ox-npm-registration-"));
  roots.push(root);
  const bin = join(root, "fake-bin");
  const packageDir = join(root, "npm", "oxct");
  const packed = join(root, "packed", "package");
  const scripts = join(root, "tools", "scripts");
  for (const dir of [bin, packageDir, packed, scripts]) mkdirSync(dir, { recursive: true });
  const manifest = {
    name: "@ox-content/cli",
    version,
    dependencies: { "@ox-content/vite-plugin": "workspace:*" },
  };
  writeFileSync(join(packageDir, "package.json"), JSON.stringify(manifest));
  writeFileSync(
    join(packed, "package.json"),
    JSON.stringify({
      ...manifest,
      dependencies: { "@ox-content/vite-plugin": version },
    }),
  );
  writeFileSync(join(scripts, "bootstrap-npm-package.mjs"), script);
  const callLog = join(root, "calls.jsonl");
  const fakeVp = join(bin, "vp");
  writeFileSync(
    fakeVp,
    `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const args = process.argv.slice(2);
const call = { args };
if (args.includes('stage')) {
  call.manifest = JSON.parse(fs.readFileSync(path.join(args[args.indexOf('publish') + 1], 'package.json')));
}
fs.appendFileSync(process.env.REGISTRATION_CALL_LOG, JSON.stringify(call) + '\\n');
if (args.includes('pack')) {
  const destination = args[args.indexOf('--pack-destination') + 1];
  const tarball = path.join(destination, 'cli.tgz');
  const result = spawnSync('tar', ['-czf', tarball, '-C', path.join(process.env.REGISTRATION_ROOT, 'packed'), 'package']);
  if (result.status !== 0) process.exit(result.status || 1);
  fs.appendFileSync(process.env.REGISTRATION_DIRS, destination + '\\n');
  console.log(tarball);
}
`,
  );
  chmodSync(fakeVp, 0o755);
  return { root, bin, callLog };
}

function runBootstrap(fixture: ReturnType<typeof createFixture>, flag?: string) {
  const dirs = join(fixture.root, "pack-dirs.txt");
  writeFileSync(dirs, "");
  const result = spawnSync(
    process.execPath,
    [
      join(fixture.root, "tools", "scripts", "bootstrap-npm-package.mjs"),
      "npm/oxct",
      ...(flag ? [flag] : []),
    ],
    {
      cwd: fixture.root,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${fixture.bin}${process.platform === "win32" ? ";" : ":"}${process.env.PATH}`,
        REGISTRATION_ROOT: fixture.root,
        REGISTRATION_CALL_LOG: fixture.callLog,
        REGISTRATION_DIRS: dirs,
      },
    },
  );
  for (const dir of readFileSync(dirs, "utf8").trim().split("\n").filter(Boolean)) {
    roots.push(dir);
  }
  return result;
}

function readCalls(fixture: ReturnType<typeof createFixture>) {
  return readFileSync(fixture.callLog, "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
}
