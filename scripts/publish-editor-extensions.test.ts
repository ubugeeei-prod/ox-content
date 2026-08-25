import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { describe, expect, it } from "vite-plus/test";

const scriptPath = resolve("scripts/publish-editor-extensions.ts");

describe("publish editor extensions", () => {
  it("skips VS Code Marketplace publish for prerelease extension versions", () => {
    const fixture = createFixture();
    const result = runPublisher(fixture, "vscode-marketplace", {
      GITHUB_REF_NAME: "v3.0.0-alpha.7",
      VSCE_PAT: "test-token",
    });

    expect(result.status).toBe(0);
    expect(result.stdout).toContain("Skipping VS Code Marketplace publish");
    expect(result.stdout).toContain("3.0.0-alpha.7");
    expect(existsSync(fixture.callLog)).toBe(false);
  });

  it("keeps Open VSX prerelease publishing and stable Marketplace publishing enabled", () => {
    const openVsxFixture = createFixture();
    const openVsx = runPublisher(openVsxFixture, "open-vsx", {
      GITHUB_REF_NAME: "v3.0.0-alpha.7",
      OVSX_PAT: "test-token",
    });

    expect(openVsx.status).toBe(0);
    expect(readFileSync(openVsxFixture.callLog, "utf8")).toContain("ovsx");

    const marketplaceFixture = createFixture();
    const marketplace = runPublisher(marketplaceFixture, "vscode-marketplace", {
      GITHUB_REF_NAME: "v3.0.0",
      VSCE_PAT: "test-token",
    });

    expect(marketplace.status).toBe(0);
    expect(readFileSync(marketplaceFixture.callLog, "utf8")).toContain("@vscode/vsce");
  });
});

function createFixture() {
  const root = mkdtempSync(join(tmpdir(), "ox-editor-publish-"));
  const binDir = join(root, "bin");
  const vsixDir = join(root, "dist", "vscode");
  const callLog = join(root, "vp-calls.log");

  mkdirSync(binDir);
  mkdirSync(vsixDir, { recursive: true });
  writeFileSync(join(vsixDir, "vscode-ox-content-linux-x64.vsix"), "");

  const vpPath = join(binDir, "vp");
  writeFileSync(
    vpPath,
    [
      "#!/usr/bin/env node",
      "const fs = require('node:fs');",
      "fs.appendFileSync(process.env.VP_CALL_LOG, JSON.stringify(process.argv.slice(2)) + '\\n');",
      "process.exit(0);",
    ].join("\n"),
  );
  chmodSync(vpPath, 0o755);

  return { root, binDir, callLog };
}

function runPublisher(
  fixture: ReturnType<typeof createFixture>,
  registry: "open-vsx" | "vscode-marketplace",
  env: Record<string, string>,
) {
  return spawnSync(process.execPath, [scriptPath, registry], {
    cwd: fixture.root,
    encoding: "utf8",
    env: {
      ...process.env,
      ...env,
      PATH: `${fixture.binDir}${process.platform === "win32" ? ";" : ":"}${process.env.PATH}`,
      VP_CALL_LOG: fixture.callLog,
    },
  });
}
