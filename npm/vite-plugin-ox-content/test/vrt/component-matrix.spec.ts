import { readFile, rm, mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import { bundleBrowserClient } from "../../../ox-content-code-play/src/bundle-browser";
import { resolveLanguage } from "../../../ox-content-code-play/src/catalog";
import { resolveCodePlayOptions } from "../../../ox-content-code-play/src/config";
import { enhancePlayHtml } from "../../../ox-content-code-play/src/html";
import { rewritePlayFences } from "../../../ox-content-code-play/src/markdown";
import { decodePayload, encodePayload } from "../../../ox-content-code-play/src/payload";
import { payloadFromFence } from "../../../ox-content-code-play/src/payload-factory";
import { highContrast } from "../../../theme-color/high-contrast/src/index";
import { kiosk } from "../../../theme/kiosk/src/index";
import { transformAllPlugins } from "../../src/plugins";
import { resetTabGroupCounter } from "../../src/plugins/tabs";
import { generateHtmlPage } from "../../src/ssg";
import { resolveTheme } from "../../src/theme";
import { transformMarkdown } from "../../src/transform";
import type { ResolvedThemeConfig } from "../../src/theme";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";

const repoRoot = fileURLToPath(new URL("../../../..", import.meta.url));
const docsRoot = path.join(repoRoot, "docs");
const matrixPath = path.join(docsRoot, "content/built-in/component-matrix.md");
let browserClient: string | undefined;
let browserClientDir: string | undefined;

test.afterAll(async () => {
  if (browserClientDir) {
    await rm(browserClientDir, { recursive: true, force: true });
  }
});

async function readCodePlayClient(): Promise<string> {
  if (browserClient) {
    return browserClient;
  }
  browserClientDir = await mkdtemp(path.join(tmpdir(), "ox-component-matrix-vrt-"));
  await bundleBrowserClient(browserClientDir);
  browserClient = await readFile(path.join(browserClientDir, "browser.mjs"), "utf8");
  return browserClient;
}

async function renderMatrix(theme?: ResolvedThemeConfig): Promise<string> {
  resetTabGroupCounter();
  const source = await readFile(matrixPath, "utf8");
  const playOptions = resolveCodePlayOptions({
    languages: {
      javascript: true,
      typescript: { execute: true, typecheck: true },
    },
  });
  const withPlayPayloads = rewritePlayFences(source, (fence) => {
    const definition = resolveLanguage(fence.language);
    if (!definition || !playOptions.languages.has(definition.id)) {
      return null;
    }
    return encodePayload(payloadFromFence(fence, playOptions));
  });
  const result = await transformMarkdown(
    withPlayPayloads,
    matrixPath,
    createDocsResolvedOptions({
      highlight: true,
      codeAnnotations: {
        enabled: true,
        notation: "both",
        metaKey: "annotate",
        defaultLineNumbers: false,
      },
      codeImports: { enabled: true, rootDir: docsRoot },
      containers: { enabled: true, types: {} },
      fileTree: { enabled: true, defaultOpen: true, icons: true },
      codeGroups: { enabled: true },
      keyboardKeys: { enabled: true, aliases: {}, style: "words" },
      math: { enabled: true },
      mermaid: false,
      embeds: {
        github: false,
        openGraph: false,
        pm: true,
        spotify: false,
        appleMusic: false,
        speakerDeck: false,
        audio: true,
        video: false,
        stackBlitz: false,
        twitter: false,
        bluesky: true,
        webContainer: false,
      },
    }),
  );
  const ssgContent = await transformAllPlugins(result.html, {
    github: false,
    openGraph: false,
    pm: true,
    audio: true,
    bluesky: true,
    mermaid: false,
  });
  const content = enhancePlayHtml(ssgContent, { decodePayload, encodePayload });
  const html = await generateHtmlPage(
    {
      title: "Component Matrix",
      description:
        "Authoring API, generated HTML contracts, accessibility behavior, theme hooks, and runtime notes.",
      content,
      toc: result.toc,
      frontmatter: {},
      path: "/built-in/component-matrix",
      href: "/built-in/component-matrix/index.html",
    },
    [
      {
        title: "Guide",
        items: [{ title: "Component Matrix", path: "/built-in/component-matrix", href: "." }],
      },
    ],
    "Ox Content",
    "/",
    undefined,
    theme,
    "en",
  );
  return html.replace(
    "</body>",
    `<script type="module">${await readCodePlayClient()}</script></body>`,
  );
}

async function openMatrix(page: Page, theme?: ResolvedThemeConfig) {
  await page.setViewportSize({ width: 1280, height: 8200 });
  await page.setContent(await renderMatrix(theme), { waitUntil: "domcontentloaded" });
  await page.addStyleTag({
    content: `
      * { caret-color: transparent !important; }
      .content { animation: none !important; }
      audio.ox-audio {
        display: block !important;
        height: 2.5rem !important;
        visibility: hidden !important;
      }
    `,
  });
  await page.evaluate(() => {
    document.documentElement.setAttribute("data-theme", "dark");
  });
  await page.waitForLoadState("load");
  await page.waitForFunction(() => document.fonts.status === "loaded");
  await page.locator(".ox-code-play").waitFor({ state: "visible" });
}

async function expectMatrixContracts(page: Page) {
  await expect(page.locator(".content")).toContainText("Authoring Contracts");
  await expect(page.locator(".ox-callout.ox-callout--note")).toBeVisible();
  await expect(page.locator("details.ox-container--details")).toHaveCount(2);
  await expect(page.locator(".ox-tabs")).toHaveCount(5);
  await expect(page.locator(".ox-file-tree")).toHaveCount(2);
  await expect(page.locator(".ox-code-block--annotated")).toHaveCount(2);
  await expect(page.locator(".ox-math-inline")).toBeVisible();
  await expect(page.locator(".ox-bluesky")).toBeVisible();
  await expect(page.locator("audio.ox-audio")).toHaveAttribute("aria-label", "Intro audio");
  await expect(page.locator("[data-ox-code-play] .ox-code-play")).toBeVisible();
  await expect(page.locator(".ox-code-play__status")).toHaveAttribute("aria-live", "polite");
  await expect(page.locator(".search-button")).toBeVisible();

  const metrics = await page.locator(".content").evaluate((content) => {
    const width = content.getBoundingClientRect().width;
    return {
      width,
      wideChildren: [...content.querySelectorAll<HTMLElement>(":scope > *")].filter(
        (child) => child.getBoundingClientRect().width > width + 1,
      ).length,
      nestedContainers: content.querySelectorAll(".ox-container .ox-container").length,
      nestedTabs: content.querySelectorAll(".ox-tabs .ox-tabs").length,
      hasMermaidOutput:
        content.querySelectorAll(".ox-mermaid, pre[data-language='mermaid'], code.language-mermaid")
          .length > 0,
    };
  });

  expect(metrics.wideChildren).toBe(0);
  expect(metrics.nestedContainers).toBe(0);
  expect(metrics.nestedTabs).toBe(0);
  expect(metrics.hasMermaidOutput).toBe(true);
}

async function contentClip(page: Page) {
  const box = await page.locator(".content").boundingBox();
  if (!box) {
    throw new Error("Component matrix content box was not rendered");
  }

  return {
    x: Math.round(box.x),
    y: Math.round(box.y),
    width: Math.round(box.width),
    height: 7600,
  };
}

test("component matrix renders under the default theme", async ({ page }) => {
  await openMatrix(page);
  await expectMatrixContracts(page);

  await expect(page).toHaveScreenshot("component-matrix-default.png", {
    animations: "disabled",
    caret: "hide",
    clip: await contentClip(page),
    maxDiffPixelRatio: 0.06,
    scale: "css",
  });
});

test("component matrix renders under the dense kiosk theme", async ({ page }) => {
  await openMatrix(page, resolveTheme([kiosk, highContrast]));
  await expectMatrixContracts(page);

  await expect(page).toHaveScreenshot("component-matrix-kiosk.png", {
    animations: "disabled",
    caret: "hide",
    clip: await contentClip(page),
    maxDiffPixelRatio: 0.06,
    scale: "css",
  });
});
