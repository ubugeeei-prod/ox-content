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

const scriptPath = resolve("tools/scripts/publish-editor-extensions.ts");

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
      ACTIONS_ID_TOKEN_REQUEST_URL: "https://example.invalid/oidc",
      ACTIONS_ID_TOKEN_REQUEST_TOKEN: "test-oidc-request-token",
    });

    expect(openVsx.status).toBe(0);
    expect(readFileSync(openVsxFixture.callLog, "utf8")).toContain("ovsx@1.2.0");

    const marketplaceFixture = createFixture();
    const marketplace = runPublisher(marketplaceFixture, "vscode-marketplace", {
      GITHUB_REF_NAME: "v3.0.0",
      VSCE_PAT: "test-token",
    });

    expect(marketplace.status).toBe(0);
    expect(readFileSync(marketplaceFixture.callLog, "utf8")).toContain("@vscode/vsce");
  });

  it("uses OIDC for every Open VSX target and ignores a configured PAT", () => {
    const fixture = createFixture();
    writeFileSync(join(fixture.root, "dist", "vscode", "vscode-ox-content-darwin-arm64.vsix"), "");
    const result = runPublisher(fixture, "open-vsx", {
      GITHUB_REF_NAME: "v3.3.0",
      ACTIONS_ID_TOKEN_REQUEST_URL: "https://example.invalid/oidc",
      ACTIONS_ID_TOKEN_REQUEST_TOKEN: "test-oidc-request-token",
      OVSX_PAT: "unused-test-token",
    });

    expect(result.status).toBe(0);
    const calls = readFileSync(fixture.callLog, "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
    expect(calls).toHaveLength(2);
    for (const call of calls) {
      expect(call.args).toContain("--trusted-publishing");
      expect(call.args).not.toContain("-p");
      expect(call.hasOpenVsxPat).toBe(false);
    }
  });

  it("fails Open VSX publishing without OIDC credentials", () => {
    const fixture = createFixture();
    const result = runPublisher(fixture, "open-vsx", { GITHUB_REF_NAME: "v3.3.0" });

    expect(result.status).toBe(1);
    expect(result.stderr).toContain("requires GitHub Actions OIDC credentials");
    expect(existsSync(fixture.callLog)).toBe(false);
  });

  it("allows packaging-only dispatch without OIDC credentials", () => {
    const fixture = createFixture();
    const result = runPublisher(fixture, "open-vsx", {
      GITHUB_EVENT_NAME: "workflow_dispatch",
      PUBLISH_FROM_DISPATCH: "false",
    });

    expect(result.status).toBe(0);
    expect(result.stdout).toContain("Packaging-only dispatch");
    expect(existsSync(fixture.callLog)).toBe(false);
  });

  it.each(["stdout", "stderr"])(
    "fails immediately with setup instructions when Open VSX rejects trust on %s",
    (stream) => {
      const fixture = createFixture();
      writeFileSync(
        join(fixture.root, "dist", "vscode", "vscode-ox-content-darwin-arm64.vsix"),
        "",
      );
      const result = runPublisher(fixture, "open-vsx", {
        ACTIONS_ID_TOKEN_REQUEST_URL: "https://example.invalid/oidc",
        ACTIONS_ID_TOKEN_REQUEST_TOKEN: "test-oidc-request-token",
        VP_FAILURES: "6",
        VP_FAILURE_STREAM: stream,
        VP_FAILURE_MESSAGE: "No trusted publisher matches the presented token.",
      });

      expect(result.status).toBe(1);
      expect(readFileSync(fixture.callLog, "utf8").trim().split("\n")).toHaveLength(1);
      expect(result[stream as "stdout" | "stderr"]).toContain(
        "No trusted publisher matches the presented token.",
      );
      expect(result.stderr).toContain("ubugeeei.vscode-ox-content");
      expect(result.stderr).toContain("ubugeeei-prod/ox-content");
      expect(result.stderr).toContain("publish-editors.yml, environment open-vsx");
      expect(result.stderr).toContain("https://open-vsx.org/user-settings/trusted-publishers");
      expect(result.stderr).toContain("rerun only the failed Open VSX job");
      expect(result.stdout).not.toContain("retrying");
    },
  );

  it.each(["open-vsx", "vscode-marketplace"] as const)(
    "retries temporary %s failures and continues publishing",
    (registry) => {
      const fixture = createFixture();
      const result = runPublisher(fixture, registry, {
        VSCE_PAT: "test-token",
        ACTIONS_ID_TOKEN_REQUEST_URL: "https://example.invalid/oidc",
        ACTIONS_ID_TOKEN_REQUEST_TOKEN: "test-oidc-request-token",
        VP_FAILURES: "1",
        VP_FAILURE_MESSAGE: "503 Service Unavailable",
      });

      expect(result.status).toBe(0);
      expect(readFileSync(fixture.callLog, "utf8").trim().split("\n")).toHaveLength(2);
      expect(result.stderr).toContain("503 Service Unavailable");
      expect(result.stdout).toContain("attempt 1/6, retrying in 30s");
      expect(result.stderr).not.toContain("trusted publisher setup is incomplete");
    },
  );

  it("still fails after exhausting retries for a temporary Open VSX failure", () => {
    const fixture = createFixture();
    const result = runPublisher(fixture, "open-vsx", {
      ACTIONS_ID_TOKEN_REQUEST_URL: "https://example.invalid/oidc",
      ACTIONS_ID_TOKEN_REQUEST_TOKEN: "test-oidc-request-token",
      VP_FAILURES: "6",
      VP_FAILURE_MESSAGE: "503 Service Unavailable",
    });

    expect(result.status).toBe(1);
    expect(readFileSync(fixture.callLog, "utf8").trim().split("\n")).toHaveLength(6);
    expect(result.stderr).toContain("failed after 6 attempts");
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
      "fs.appendFileSync(process.env.VP_CALL_LOG, JSON.stringify({ args: process.argv.slice(2), hasOpenVsxPat: process.env.OVSX_PAT !== undefined }) + '\\n');",
      "const attempts = fs.readFileSync(process.env.VP_CALL_LOG, 'utf8').trim().split('\\n').length;",
      "if (attempts <= Number(process.env.VP_FAILURES ?? 0)) {",
      "  process[process.env.VP_FAILURE_STREAM ?? 'stderr'].write(process.env.VP_FAILURE_MESSAGE + '\\n');",
      "  process.exit(1);",
      "}",
      "process.exit(0);",
    ].join("\n"),
  );
  chmodSync(vpPath, 0o755);

  // Exercise the real retry loop without waiting through production backoffs.
  const skipDelayPath = join(root, "skip-delay.cjs");
  writeFileSync(
    skipDelayPath,
    "require('node:timers/promises').setTimeout = async () => {};\n" +
      "require('node:module').syncBuiltinESMExports();\n",
  );

  return { root, binDir, callLog, skipDelayPath };
}

function runPublisher(
  fixture: ReturnType<typeof createFixture>,
  registry: "open-vsx" | "vscode-marketplace",
  env: Record<string, string>,
) {
  return spawnSync(process.execPath, ["--require", fixture.skipDelayPath, scriptPath, registry], {
    cwd: fixture.root,
    encoding: "utf8",
    timeout: 10_000,
    env: {
      ...process.env,
      ACTIONS_ID_TOKEN_REQUEST_URL: "",
      ACTIONS_ID_TOKEN_REQUEST_TOKEN: "",
      OVSX_PAT: "",
      GITHUB_EVENT_NAME: "push",
      ...env,
      PATH: `${fixture.binDir}${process.platform === "win32" ? ";" : ":"}${process.env.PATH}`,
      VP_CALL_LOG: fixture.callLog,
    },
  });
}
