import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";
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
    },
    gfm: true,
    footnotes: true,
    tables: true,
    taskLists: true,
    strikethrough: true,
    highlight: true,
    highlightTheme: "vitesse-dark",
    highlightLangs: [],
    codeAnnotations: {
      enabled: true,
      notation: "vitepress",
      metaKey: "annotate",
      defaultLineNumbers: false,
    },
    mermaid: false,
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

test("renders VitePress-style code highlighting", async ({ page }) => {
  const [sansFont, monoFont] = await Promise.all([
    readFontDataUrl("KaTeX_SansSerif-Regular.woff2"),
    readFontDataUrl("KaTeX_Typewriter-Regular.woff2"),
  ]);
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
