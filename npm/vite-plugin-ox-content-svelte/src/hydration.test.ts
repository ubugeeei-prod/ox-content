import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { svelte as sveltePlugin } from "@sveltejs/vite-plugin-svelte";
import { compile } from "svelte/compiler";
import { render } from "svelte/server";
import { createServer, type ViteDevServer } from "vite";
import { chromium, type Browser } from "playwright";
import { describe, expect, it } from "vite-plus/test";
import { transformMarkdownWithSvelte } from "./transform";
import type { Component } from "svelte";
import type { ResolvedSvelteOptions } from "./types";

declare global {
  interface Window {
    __destroy: () => void | Promise<void>;
    __pageHydrated?: boolean;
  }
}

const DIRNAME = path.dirname(fileURLToPath(import.meta.url));
const PACKAGE_ROOT = path.resolve(DIRNAME, "..");
const ISLANDS_ENTRY = path.resolve(DIRNAME, "../../ox-content-islands/src/index.ts");
const ISLANDS_ROOT = path.resolve(DIRNAME, "../../ox-content-islands");

describe("Svelte island hydration", () => {
  it("hydrates server-rendered MDX islands without duplicating markup", async () => {
    const root = await mkdtemp(path.join(PACKAGE_ROOT, ".tmp-hydration-"));
    let browser: Browser | undefined;
    let server: ViteDevServer | undefined;

    try {
      await mkdir(path.join(root, "docs"), { recursive: true });
      await writeFile(path.join(root, "docs", "Counter.svelte"), COUNTER_SVELTE);
      const serverCounterPath = path.join(root, "docs", "Counter.server.mjs");
      await writeFile(
        serverCounterPath,
        compile(COUNTER_SVELTE, {
          filename: "Counter.svelte",
          generate: "server",
          runes: true,
        }).js.code,
      );
      const serverCounter = (await import(
        `${pathToFileURL(serverCounterPath).href}?t=${Date.now()}`
      )) as { default: Component<Record<string, unknown>> };

      server = await createServer({
        root,
        appType: "mpa",
        logLevel: "silent",
        plugins: [sveltePlugin()],
        resolve: {
          alias: {
            "@ox-content/islands": ISLANDS_ENTRY,
          },
        },
        server: {
          host: "127.0.0.1",
          port: 0,
          strictPort: false,
          fs: {
            allow: [root, PACKAGE_ROOT, ISLANDS_ROOT],
          },
        },
      });
      await server.listen();

      const markdown = [
        "import Counter from './Counter.svelte'",
        "",
        '<Counter initial={1} label="Clicks">',
        "  <strong>ready</strong>",
        "</Counter>",
      ].join("\n");
      const documentPath = path.join(root, "docs", "page.mdx");
      const options = createOptions({
        components: {},
        root,
        renderIsland: async (_name, props, _filePath, slotHtml) => {
          const componentProps = { ...props };
          if (slotHtml?.trim()) {
            componentProps.children = (renderer: { push(html: string): void }) => {
              renderer.push(`<div>${slotHtml}</div>`);
            };
          }
          return render(serverCounter.default, { props: componentProps }).html;
        },
      });

      const ssrResult = await transformMarkdownWithSvelte(markdown, documentPath, {
        ...options,
        ssr: true,
      });
      const clientResult = await transformMarkdownWithSvelte(markdown, documentPath, {
        ...options,
        ssr: false,
      });
      expect(ssrResult.code).toContain('data-ox-ssr=\\"true\\"');
      expect(clientResult.code).toContain("element.dataset.oxSsr === 'true' ? hydrate : mount");

      await writeFile(path.join(root, "docs", "page-server.mjs"), ssrResult.code);
      await writeFile(path.join(root, "docs", "page-client.mjs"), clientResult.code);
      await writeFile(
        path.join(root, "docs", "entry.js"),
        [
          "import Page from './page-client.mjs';",
          "import { hydrate, unmount } from 'svelte';",
          "const instance = hydrate(Page, { target: document.getElementById('app') });",
          "window.__destroy = () => unmount(instance);",
          "window.__pageHydrated = true;",
        ].join("\n"),
      );

      const pageModule = (await server.ssrLoadModule("/docs/page-server.mjs")) as {
        default: Component<Record<string, unknown>>;
      };
      const pageHtml = render(pageModule.default).html;
      expect(pageHtml).toContain('data-testid="counter"');
      expect(pageHtml).toContain('data-testid="slot"');
      expect(countOccurrences(pageHtml, 'data-testid="counter"')).toBe(1);
      await writeFile(
        path.join(root, "docs", "index.html"),
        [
          "<!doctype html><html><body>",
          `<div id="app">${pageHtml}</div>`,
          '<script type="module" src="/docs/entry.js"></script>',
          "</body></html>",
        ].join(""),
      );

      browser = await chromium.launch({
        channel: process.env.CI ? "chrome" : undefined,
        headless: true,
      });
      const page = await browser.newPage();
      const browserDiagnostics: string[] = [];
      page.on("console", (message) => {
        browserDiagnostics.push(`${message.type()}: ${message.text()}`);
      });
      page.on("pageerror", (error) => {
        browserDiagnostics.push(`pageerror: ${error.message}`);
      });
      const response = await page.goto(`${serverUrl(server)}/docs/index.html`);
      expect(response?.status()).toBe(200);
      await page.waitForFunction(() => window.__pageHydrated === true);
      await page.waitForFunction(() => {
        const island = document.querySelector("[data-ox-island]");
        return island?.getAttribute("data-ox-hydrated") === "true";
      });
      await page.waitForFunction(
        () => document.querySelectorAll('[data-testid="counter"]').length === 1,
      );
      await page.waitForFunction(
        () => document.querySelectorAll('[data-testid="slot"]').length === 1,
      );
      await page.locator('[data-testid="counter"]').waitFor({ state: "attached" });
      await page.locator('[data-testid="slot"]').waitFor({ state: "attached" });

      const counterCount = await page.locator('[data-testid="counter"]').count();
      if (counterCount !== 1) {
        throw new Error(
          [
            `Expected one hydrated counter, received ${counterCount}.`,
            `Console: ${browserDiagnostics.join("\n")}`,
            `HTML: ${await page.content()}`,
          ].join("\n"),
        );
      }
      const islandState = await page.locator("[data-ox-island]").evaluate((element) => ({
        innerHTML: element.innerHTML,
        oxContent: (element as HTMLElement).dataset.oxContent,
      }));
      expect(islandState.oxContent).toContain("<p><strong>ready</strong></p>");
      expect(islandState.innerHTML).toContain('data-testid="slot"');
      expect(await page.locator('[data-testid="slot"]').count()).toBe(1);
      expect(await page.locator('[data-ox-island][data-ox-ssr="true"]').count()).toBe(1);
      expect(await page.locator('[data-ox-island][data-ox-hydrated="true"]').count()).toBe(1);
      expect(await page.locator('[data-testid="button"]').textContent()).toBe("Clicks: 1");
      await page.locator('[data-testid="button"]').click();
      await page.waitForFunction(
        () => document.querySelector('[data-testid="button"]')?.textContent === "Clicks: 2",
      );

      await page.evaluate(() => window.__destroy());
      await page.waitForFunction(() => document.querySelector("#app")?.childElementCount === 0);
    } finally {
      await browser?.close();
      await server?.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("keeps non-SSR islands on the mount path", async () => {
    const result = await transformMarkdownWithSvelte(
      "import Counter from './Counter.svelte'\n\n<Counter initial={1} />\n",
      "/repo/docs/page.mdx",
      createOptions({ components: {} }),
    );

    expect(result.code).toContain("element.dataset.oxSsr === 'true' ? hydrate : mount");
    expect(result.code).not.toContain('data-ox-ssr="true"');
  });
});

function createOptions(overrides: Partial<ResolvedSvelteOptions> = {}): ResolvedSvelteOptions {
  return {
    srcDir: "docs",
    outDir: "dist",
    base: "/",
    extensions: [".md", ".markdown", ".mdx"],
    gfm: true,
    autolinks: true,
    frontmatter: true,
    toc: true,
    tocMaxDepth: 3,
    codeAnnotations: { enabled: false, metaKey: "annotate" },
    components: { Counter: "./docs/Counter.svelte" },
    runes: true,
    embeds: { github: false, openGraph: false },
    root: "/repo",
    mdxDocumentProps: false,
    ...overrides,
  } as ResolvedSvelteOptions;
}

function serverUrl(server: ViteDevServer): string {
  const url = server.resolvedUrls?.local[0];
  if (!url) throw new Error("Vite server did not expose a local URL.");
  return url.replace(/\/$/, "");
}

function countOccurrences(value: string, needle: string): number {
  return value.split(needle).length - 1;
}

const COUNTER_SVELTE = `
<script>
  let { initial = 0, label = 'Clicks', children } = $props();
  let count = $state(initial);
</script>

<section data-testid="counter">
  <button data-testid="button" onclick={() => count += 1}>{label}: {count}</button>
  {#if children}
    <div data-testid="slot">{@render children()}</div>
  {/if}
</section>
`;
