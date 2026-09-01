import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";

const bin = resolve(dirname(fileURLToPath(import.meta.url)), "..", "bin", "oxct.mjs");

describe("oxct CLI", () => {
  it("prints the top-level help without loading the native binding", () => {
    const result = spawnSync(process.execPath, [bin, "--help"], { encoding: "utf8" });

    expect(result.status).toBe(0);
    expect(result.stdout).toContain("oxct <command>");
    expect(result.stdout).toContain("i18n <command>");
    expect(result.stdout).toContain("link-check");
    expect(result.stdout).toContain("migrate vitepress");
    expect(result.stdout).toContain("mdc-check");
    expect(result.stdout).toContain("lsp");
    expect(result.stdout).toContain("og-preview");
  });

  it("prints i18n help without loading the native binding", () => {
    const result = spawnSync(process.execPath, [bin, "i18n", "--help"], { encoding: "utf8" });

    expect(result.status).toBe(0);
    expect(result.stdout).toContain("oxct i18n check");
    expect(result.stdout).toContain("oxct i18n validate");
  });

  it.each([
    ["link-check", "oxct link-check"],
    ["mdc-check", "oxct mdc-check"],
    ["lsp", "oxct lsp"],
    ["migrate", "oxct migrate vitepress"],
    ["og-preview", "oxct og-preview"],
  ])("prints %s help without loading implementation dependencies", (command, expected) => {
    const result = spawnSync(process.execPath, [bin, command, "--help"], { encoding: "utf8" });

    expect(result.status).toBe(0);
    expect(result.stdout).toContain(expected);
  });

  it("prints nested VitePress migration help with the oxct command path", () => {
    const result = spawnSync(process.execPath, [bin, "migrate", "vitepress", "--help"], {
      encoding: "utf8",
    });

    expect(result.status).toBe(0);
    expect(result.stdout).toContain("oxct migrate vitepress [config]");
  });
});
