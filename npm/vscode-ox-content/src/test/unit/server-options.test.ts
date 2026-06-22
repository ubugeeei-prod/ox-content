/**
 * Pure-node unit tests for the server-command precedence and
 * initialization-options mapping extracted from `config.ts`.
 *
 * These run under vitest with no VS Code Electron host, exercising the
 * branches `extension.test.ts` cannot reach without a real filesystem
 * of fake binaries or mutated `process.env`.
 */

import { describe, expect, it } from "vitest";

import { buildInitializationOptions, selectServerCommand } from "../../internal/server-options";

const CARGO_FALLBACK = {
  command: "cargo",
  args: ["run", "-p", "ox_content_lsp", "--bin", "ox-content-lsp"],
};

describe("selectServerCommand", () => {
  const localCandidates = [
    "/work/target/debug/ox-content-lsp",
    "/work/target/release/ox-content-lsp",
  ];

  it("prefers a configured path that exists over everything else", () => {
    const result = selectServerCommand({
      configuredPath: "/custom/ox-content-lsp",
      envBinary: "/env/ox-content-lsp",
      localCandidates,
      exists: () => true,
    });
    expect(result).toEqual({ command: "/custom/ox-content-lsp", args: [] });
  });

  it("falls through to the env binary when the configured path is missing", () => {
    const result = selectServerCommand({
      configuredPath: "/custom/ox-content-lsp",
      envBinary: "/env/ox-content-lsp",
      localCandidates,
      exists: (candidate) => candidate !== "/custom/ox-content-lsp",
    });
    expect(result).toEqual({ command: "/env/ox-content-lsp", args: [] });
  });

  it("prefers the env binary over local candidates", () => {
    const result = selectServerCommand({
      envBinary: "/env/ox-content-lsp",
      localCandidates,
      exists: () => true,
    });
    expect(result).toEqual({ command: "/env/ox-content-lsp", args: [] });
  });

  it("uses the first local candidate that exists", () => {
    const result = selectServerCommand({
      localCandidates,
      // debug build is absent; release build is present.
      exists: (candidate) => candidate === "/work/target/release/ox-content-lsp",
    });
    expect(result).toEqual({ command: "/work/target/release/ox-content-lsp", args: [] });
  });

  it("falls back to cargo run when nothing exists", () => {
    const result = selectServerCommand({
      configuredPath: "/custom/ox-content-lsp",
      envBinary: "/env/ox-content-lsp",
      localCandidates,
      exists: () => false,
    });
    expect(result).toEqual(CARGO_FALLBACK);
  });

  it("ignores an empty configured path and env binary", () => {
    const result = selectServerCommand({
      configuredPath: undefined,
      envBinary: undefined,
      localCandidates: [],
      exists: () => true,
    });
    expect(result).toEqual(CARGO_FALLBACK);
  });
});

describe("buildInitializationOptions", () => {
  const resolvePath = (value: string) => `/work/${value}`;

  it("returns an empty object when nothing is configured", () => {
    const options = buildInitializationOptions({
      textlintEnabled: false,
      spacingAutoFixOnSave: false,
      resolvePath,
    });
    expect(options).toEqual({});
  });

  it("resolves the schema and mdc paths against the workspace root", () => {
    const options = buildInitializationOptions({
      schema: "schema.json",
      mdcComponents: "components.json",
      textlintEnabled: false,
      spacingAutoFixOnSave: false,
      resolvePath,
    });
    expect(options).toEqual({
      frontmatterSchema: "/work/schema.json",
      mdcComponents: "/work/components.json",
    });
  });

  it("forwards textlint only when enabled", () => {
    const disabled = buildInitializationOptions({
      textlintEnabled: false,
      textlintCommand: "pnpm exec textlint",
      spacingAutoFixOnSave: false,
      resolvePath,
    });
    // The command is still forwarded so a user can pre-seed it, but the
    // enabled flag must be absent so the server stays opted-out.
    expect(disabled.textlintEnabled).toBeUndefined();
    expect(disabled.textlintCommand).toBe("pnpm exec textlint");

    const enabled = buildInitializationOptions({
      textlintEnabled: true,
      spacingAutoFixOnSave: false,
      resolvePath,
    });
    expect(enabled.textlintEnabled).toBe(true);
  });

  it("omits empty-string settings", () => {
    const options = buildInitializationOptions({
      schema: "",
      textlintEnabled: false,
      textlintCommand: "",
      mdcComponents: "",
      spaceBetweenHalfAndFullWidth: "",
      spacingAutoFixOnSave: false,
      resolvePath,
    });
    expect(options).toEqual({});
  });

  it("forwards spacing options", () => {
    const options = buildInitializationOptions({
      textlintEnabled: false,
      spaceBetweenHalfAndFullWidth: "require",
      spacingAutoFixOnSave: true,
      resolvePath,
    });
    expect(options).toEqual({
      spaceBetweenHalfAndFullWidth: "require",
      spacingAutoFixOnSave: true,
    });
  });
});
