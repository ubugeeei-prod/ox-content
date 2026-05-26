/**
 * Pure-node unit tests for the path helpers extracted from `config.ts`.
 *
 * These run under vitest with no VS Code Electron host, so they cover
 * the branches `extension.test.ts` cannot exercise without monkey-patching
 * `process.platform` (Windows binary name) or shipping fake filesystems
 * (alternative server.path locations).
 */

import * as path from "node:path";
import { describe, expect, it } from "vitest";

import {
  localServerBinaryCandidates,
  resolveFilePath,
  serverBinaryName,
} from "../../internal/paths";

describe("resolveFilePath", () => {
  it("returns absolute paths unchanged regardless of workspace root", () => {
    const abs = path.resolve("/tmp/foo/bar.json");
    expect(resolveFilePath(abs, "/anywhere")).toBe(abs);
    expect(resolveFilePath(abs)).toBe(abs);
  });

  it("joins relative paths against the workspace root when one is provided", () => {
    expect(resolveFilePath("./schema.json", "/work")).toBe(path.join("/work", "./schema.json"));
    expect(resolveFilePath("config/schema.json", "/work")).toBe(
      path.join("/work", "config/schema.json"),
    );
  });

  it("returns the raw relative value when no workspace root is provided", () => {
    // The caller is expected to fall through to `fs.existsSync` against
    // process.cwd() in that case; we should not silently absolutify.
    expect(resolveFilePath("schema.json")).toBe("schema.json");
  });
});

describe("serverBinaryName", () => {
  it("returns the .exe suffix on Windows", () => {
    expect(serverBinaryName("win32")).toBe("ox-content-lsp.exe");
  });

  it("returns the bare binary name on every POSIX-shaped platform", () => {
    for (const platform of ["darwin", "linux", "freebsd", "openbsd", "aix"] as const) {
      expect(serverBinaryName(platform)).toBe("ox-content-lsp");
    }
  });
});

describe("localServerBinaryCandidates", () => {
  it("orders workspace debug, workspace release, then the bundled binary", () => {
    const candidates = localServerBinaryCandidates({
      workspaceRoot: "/work",
      extensionPath: "/ext",
      platform: "linux",
    });
    expect(candidates).toEqual([
      path.join("/work", "target", "debug", "ox-content-lsp"),
      path.join("/work", "target", "release", "ox-content-lsp"),
      path.join("/ext", "bin", "ox-content-lsp"),
    ]);
  });

  it("uses the .exe suffix on Windows for every candidate", () => {
    const candidates = localServerBinaryCandidates({
      workspaceRoot: "C:\\work",
      extensionPath: "C:\\ext",
      platform: "win32",
    });
    for (const candidate of candidates) {
      expect(candidate.endsWith("ox-content-lsp.exe")).toBe(true);
    }
  });

  it("falls back to the bundled binary when there is no workspace root", () => {
    const candidates = localServerBinaryCandidates({
      extensionPath: "/ext",
      platform: "darwin",
    });
    expect(candidates).toEqual([path.join("/ext", "bin", "ox-content-lsp")]);
  });
});
