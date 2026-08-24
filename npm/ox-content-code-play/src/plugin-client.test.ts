import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import {
  assertBrowserClientSource,
  isAutoHydratingClient,
  isStandaloneBrowserClient,
  resolveClientFile,
} from "./plugin-client";

describe("plugin client emission", () => {
  it("prefers the auto-hydrating browser entry over hydrate", () => {
    const dir = mkdtempSync(path.join(tmpdir(), "ox-code-play-client-"));
    writeFileSync(path.join(dir, "browser.mjs"), "bootCodePlay();\n");
    writeFileSync(path.join(dir, "hydrate.mjs"), "export function hydrateCodePlay() {}\n");
    expect(resolveClientFile(pathToFileURL(path.join(dir, "plugin.mjs")).href)).toMatch(
      /browser\.mjs$/,
    );
  });

  it("rejects a chunked hydrate module as the published SSG client", () => {
    const hydrate = `export { hydrateCodePlay } from "./hydrate2.mjs";\n`;
    expect(isAutoHydratingClient(hydrate)).toBe(false);
    expect(isStandaloneBrowserClient(hydrate)).toBe(false);
    expect(() => assertBrowserClientSource(hydrate)).toThrow(/bootCodePlay/);
  });

  it("accepts a standalone boot entry", () => {
    const source =
      'function bootCodePlay() {}\ndocument.addEventListener("DOMContentLoaded", bootCodePlay);\n';
    expect(isAutoHydratingClient(source)).toBe(true);
    expect(isStandaloneBrowserClient(source)).toBe(true);
    expect(assertBrowserClientSource(source)).toBe(source);
  });
});
