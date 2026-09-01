import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";

const bin = resolve(dirname(fileURLToPath(import.meta.url)), "..", "bin", "oxct.mjs");

describe("oxct CLI", () => {
  it("prints the top-level help without loading the native binding", () => {
    const result = spawnSync(process.execPath, [bin, "--help"], { encoding: "utf8" });

    expect(result.status).toBe(0);
    expect(result.stdout).toContain("oxct i18n <command>");
    expect(result.stdout).toContain("i18n check");
  });

  it("prints i18n help without loading the native binding", () => {
    const result = spawnSync(process.execPath, [bin, "i18n", "--help"], { encoding: "utf8" });

    expect(result.status).toBe(0);
    expect(result.stdout).toContain("oxct i18n check");
    expect(result.stdout).toContain("oxct i18n validate");
  });
});
