import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { bundleBrowserClient } from "./bundle-browser";
import { assertBrowserClientSource } from "./plugin-client";

describe("browser client bundle", () => {
  it("inlines the widget runtime and calls bootCodePlay", async () => {
    const outDir = await mkdtemp(path.join(tmpdir(), "ox-code-play-browser-"));
    try {
      await bundleBrowserClient(outDir);
      const source = await readFile(path.join(outDir, "browser.mjs"), "utf8");
      expect(assertBrowserClientSource(source)).toContain("bootCodePlay");
      expect(source).toMatch(/hydrateCodePlay|data-ox-code-play/);
      expect(source).toContain("data-ox-action");
      expect(source).toContain("cancel");
      expect(source).toContain("allow-scripts");
      expect(source).not.toMatch(/allow-same-origin/);
    } finally {
      await rm(outDir, { recursive: true, force: true });
    }
  }, 60_000);
});
