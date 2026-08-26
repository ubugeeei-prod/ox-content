import { readFile } from "node:fs/promises";
import { expect, test, type Page } from "@playwright/test";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import type { ResolvedOptions } from "../../src/types";

function createResolvedOptions(): ResolvedOptions {
  return {
    srcDir: "content",
    outDir: "dist",
    base: "/",
    ssg: {
      enabled: true,
      extension: ".html",
      clean: false,
      bare: false,
      siteName: "Ox Content",
      generateOgImage: false,
      lastUpdated: false,
      pagination: false,
      breadcrumbs: false,
      jsonLd: false,
      readerChrome: false,
      localeSwitcher: false,
      a11y: false,
      pageChrome: false,
    },
    gfm: true,
    footnotes: true,
    tables: true,
    taskLists: true,
    strikethrough: true,
    autolinks: true,
    highlight: true,
    codeAnnotations: {
      enabled: true,
      notation: "vitepress",
      metaKey: "annotate",
      defaultLineNumbers: false,
    },
    mermaid: false,
    math: { enabled: false },
    frontmatter: true,
    toc: true,
    tocMaxDepth: 3,
    ogImage: false,
    ogImageOptions: {
      width: 1200,
      height: 630,
      cache: true,
      concurrency: 1,
      vuePlugin: "vitejs",
    },
    transformers: [],
    docs: false,
    search: {
      enabled: true,
      limit: 10,
      prefix: true,
      placeholder: "Search documentation...",
      hotkey: "/",
    },
    ogViewer: false,
    i18n: false,
  };
}

async function readFontDataUrl(fileName: string): Promise<string> {
  const font = await readFile(new URL(`./assets/${fileName}`, import.meta.url));
  return `data:font/woff2;base64,${font.toString("base64")}`;
}

async function installVrtFonts(page: Page) {
  const [sansFont, monoFont] = await Promise.all([
    readFontDataUrl("KaTeX_SansSerif-Regular.woff2"),
    readFontDataUrl("KaTeX_Typewriter-Regular.woff2"),
  ]);
  await page.addStyleTag({
    content: `
      @font-face {
        font-family: "OxContentVrtSans";
        src: url("${sansFont}") format("woff2");
        font-style: normal;
        font-weight: 400;
      }

      @font-face {
        font-family: "OxContentVrtMono";
        src: url("${monoFont}") format("woff2");
        font-style: normal;
        font-weight: 400;
      }

      :root {
        --octc-font-sans: "OxContentVrtSans", sans-serif;
        --octc-font-mono: "OxContentVrtMono", monospace;
      }
    `,
  });
}

test("renders VitePress-style code highlighting", async ({ page }) => {
  const markdown = `# VitePress Style

## Fence Metadata

\`\`\`ts:line-numbers=12 {1,3} [config.ts]
const theme = "dark";
const mode = "docs";
console.log(theme, mode);
\`\`\`

## Inline Directives

\`\`\`ts
// [!code focus:2]
const before = true;
const after = false;
console.log("old value") // [!code --]
console.log("new value") // [!code ++]
console.warn("careful") // [!code warning]
throw new Error("boom") // [!code error]
\`\`\`
`;

  const result = await transformMarkdown(
    markdown,
    "docs/vrt-code-highlighting.md",
    createResolvedOptions(),
  );
  const html = await generateHtmlPage(
    {
      title: "VitePress Style",
      content: result.html,
      toc: result.toc,
      frontmatter: {},
      path: "/vrt-code-highlighting",
      href: "/vrt-code-highlighting/index.html",
    },
    [],
    "Ox Content",
    "/",
  );

  await page.setContent(html, { waitUntil: "load" });
  await installVrtFonts(page);
  await page.evaluate(() => {
    document.documentElement.setAttribute("data-theme", "dark");
  });
  await page.waitForFunction(() => document.fonts.status === "loaded");
  await page.locator(".content pre").last().waitFor();

  await expect(page.locator(".content")).toHaveScreenshot("vitepress-code-highlighting.png", {
    animations: "disabled",
    caret: "hide",
    maxDiffPixels: 3500,
    scale: "device",
  });
});

test("renders dense code block affordances on mobile", async ({ page }) => {
  const markdown = `# Dense Code

\`\`\`ts:line-numbers=27 :line-links=auth-loader :wrap [src/auth/load-user.ts]
export async function loadUserSession(request: Request, cache: Map<string, Promise<UserSession>>) {
  const authorization = request.headers.get("authorization") ?? request.headers.get("x-legacy-auth-token");
  const cacheKey = authorization?.replace(/^Bearer\\s+/i, "") ?? "anonymous-session-without-token";
  const previous = cache.get(cacheKey); // [!code --]
  const previous = cache.get(cacheKey) ?? fetchAndStoreUserSession(cache, cacheKey, authorization); // [!code ++]
  return previous;
}
\`\`\`
`;

  const result = await transformMarkdown(
    markdown,
    "docs/vrt-dense-code.md",
    createResolvedOptions(),
  );
  const html = await generateHtmlPage(
    {
      title: "Dense Code",
      content: result.html,
      toc: result.toc,
      frontmatter: {},
      path: "/vrt-dense-code",
      href: "/vrt-dense-code/index.html",
    },
    [],
    "Ox Content",
    "/",
    undefined,
    undefined,
    undefined,
    undefined,
    false,
    { copy: true, externalLinks: false, backToTop: false },
  );

  await page.setViewportSize({ width: 390, height: 844 });
  await page.setContent(html, { waitUntil: "load" });
  await installVrtFonts(page);
  await page.evaluate(() => {
    document.documentElement.setAttribute("data-theme", "dark");
    window.location.hash = "auth-loader-L27";
  });
  await page.waitForFunction(() => document.fonts.status === "loaded");
  await page.locator(".content pre.ox-code-block--wrap").waitFor();

  const metrics = await page.locator(".content pre.ox-code-block--wrap").evaluate((pre) => {
    if (!(pre instanceof HTMLElement)) {
      throw new Error("Missing dense code block");
    }
    const firstLine = pre.querySelector("#auth-loader-L27");
    if (!(firstLine instanceof HTMLElement)) {
      throw new Error("Missing code line target");
    }
    return {
      clientWidth: pre.clientWidth,
      firstLineBackground: getComputedStyle(firstLine).backgroundColor,
      scrollWidth: pre.scrollWidth,
      whiteSpace: getComputedStyle(pre).whiteSpace,
    };
  });

  expect(metrics.whiteSpace).toBe("pre-wrap");
  expect(metrics.scrollWidth).toBeLessThanOrEqual(metrics.clientWidth + 1);
  expect(metrics.firstLineBackground).not.toBe("rgba(0, 0, 0, 0)");

  await expect(page.locator(".content")).toHaveScreenshot("dense-code-affordances-mobile.png", {
    animations: "disabled",
    caret: "hide",
    maxDiffPixels: 3500,
    scale: "device",
  });
});
