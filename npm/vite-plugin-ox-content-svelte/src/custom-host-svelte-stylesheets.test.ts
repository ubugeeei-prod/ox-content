import fs from "node:fs/promises";
import path from "node:path";
import { chromium, type Browser } from "playwright";
import { build as viteBuild, createServer } from "vite";
import { afterEach, describe, expect, it } from "vite-plus/test";
import {
  cleanupSvelteStylesheetFixture,
  createProject,
  expectRenderedStyles,
  listen,
  readLinkedCss,
  serveDist,
  trackServer,
  viteConfig,
  wait,
  writeFile,
  writeViteConfig,
} from "./custom-host-svelte-stylesheets.fixture";

afterEach(cleanupSvelteStylesheetFixture);

describe("custom host Svelte SSR stylesheets", () => {
  it.each([
    ["official Svelte", "@sveltejs/vite-plugin-svelte"],
    ["rsvelte", "@rsvelte/vite-plugin-svelte"],
  ] as const)(
    "discovers scoped Svelte CSS in production with %s",
    async (_name, plugin) => {
      const root = await createProject("ox-custom-host-svelte-build-");
      await writeViteConfig(root, plugin);
      await viteBuild(viteConfig(root));

      const html = await fs.readFile(path.join(root, "dist", "probe", "index.html"), "utf8");
      expect(html).toContain('data-diagnostics=""');
      expect(html).toContain('data-style-content-diagnostics=""');
      expect(html).toContain('data-critical="/docs/assets/');
      expect(html).toContain("Hello");
      expect(html).not.toContain('<script type="module"');

      const css = await readLinkedCss(root, html);
      expect(css).toMatch(/color:(?:red|rgb\(255,0,0\))/u);
      expect(css).toMatch(/color:(?:#00f|blue|rgb\(0,0,255\))/u);
      expect(css).toMatch(/color:(?:green|rgb\(0,128,0\))/u);
    },
    30_000,
  );

  it("serves blocking scoped Svelte styles before client JavaScript", async () => {
    const root = await createProject("ox-custom-host-svelte-browser-");
    await writeViteConfig(root, "@sveltejs/vite-plugin-svelte");
    await viteBuild(viteConfig(root));
    const staticServer = await serveDist(root);
    let browser: Browser | undefined;

    try {
      browser = await chromium.launch({
        channel: process.env.CI ? "chrome" : undefined,
        headless: true,
      });
      const page = await browser.newPage();
      const response = await page.goto(`http://127.0.0.1:${staticServer.port}/docs/probe/`);
      expect(response?.status()).toBe(200);

      expect(await page.evaluate(() => document.scripts.length)).toBe(0);
      await expectRenderedStyles(page, "rgb(0, 0, 255)");
    } finally {
      await browser?.close();
    }
  }, 30_000);

  it.each([
    ["official Svelte", "@sveltejs/vite-plugin-svelte"],
    ["rsvelte", "@rsvelte/vite-plugin-svelte"],
  ] as const)(
    "serves and invalidates scoped Svelte CSS in dev with %s",
    async (_name, plugin) => {
      const root = await createProject("ox-custom-host-svelte-dev-");
      await writeViteConfig(root, plugin, { reloadDebounceMs: 1 });
      const server = await trackServer(createServer(viteConfig(root)));
      const listener = await listen(server);
      let browser: Browser | undefined;

      try {
        browser = await chromium.launch({
          channel: process.env.CI ? "chrome" : undefined,
          headless: true,
        });
        const page = await browser.newPage();
        const response = await page.goto(`http://127.0.0.1:${listener.port}/docs/probe`);
        const firstHtml = await page.content();
        expect(response?.status(), firstHtml).toBe(200);
        expect(firstHtml).toContain('data-diagnostics=""');
        expect(firstHtml).toContain('data-style-content-diagnostics=""');
        expect(firstHtml).toContain("Page.svelte");
        expect(firstHtml).toContain("Child.svelte");
        expect(firstHtml).toContain("SiteHeader.svelte");
        expect(firstHtml).toContain("type=style");
        expect(firstHtml).toContain("data-critical");
        expect(firstHtml).not.toContain('import Child from "@/components/Child.svelte"');
        expect(firstHtml).not.toContain("<style>.child{color:rgb(0,0,255)}</style>");
        await expectRenderedStyles(page, "rgb(0, 0, 255)");

        await writeFile(
          root,
          "src/components/Child.svelte",
          '<p class="child">child</p>\n<style>.child{color:rgb(0,128,0)}</style>\n',
        );
        server.watcher.emit("change", path.join(root, "src", "components", "Child.svelte"));
        await wait(50);

        await page.reload();
        expect((await page.locator("body").getAttribute("data-render")) ?? "").toBe("2");
        await expectRenderedStyles(page, "rgb(0, 128, 0)");
      } finally {
        await browser?.close();
      }
    },
    30_000,
  );
});
