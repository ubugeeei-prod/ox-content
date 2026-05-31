import { execFileSync } from "node:child_process";
import { readFile, stat } from "node:fs/promises";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "@playwright/test";
import type { Page, Route } from "@playwright/test";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const packageRoot = path.resolve(__dirname, "../..");
const repoRoot = path.resolve(packageRoot, "../..");
const exampleRoot = path.resolve(repoRoot, "examples/slides-vite");
const exampleDist = path.resolve(exampleRoot, "dist");
const vrtAssetsDir = path.resolve(repoRoot, "npm/vite-plugin-ox-content/test/vrt/assets");

async function readFontDataUrl(fileName: string): Promise<string> {
  const font = await readFile(path.join(vrtAssetsDir, fileName));
  return `data:font/woff2;base64,${font.toString("base64")}`;
}

async function createFontInjection(): Promise<string> {
  const [sansFont, monoFont] = await Promise.all([
    readFontDataUrl("KaTeX_SansSerif-Regular.woff2"),
    readFontDataUrl("KaTeX_Typewriter-Regular.woff2"),
  ]);

  return `
    <style>
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
        --ox-slide-font-sans: "OxContentVrtSans", sans-serif;
        --ox-slide-font-mono: "OxContentVrtMono", monospace;
      }
    </style>
  `;
}

function buildSlidesExample() {
  execFileSync("pnpm", ["--filter", "@ox-content/vite-plugin-slides", "build"], {
    cwd: repoRoot,
    stdio: "pipe",
  });
  execFileSync("pnpm", ["--filter", "slides-vite-example", "build"], {
    cwd: repoRoot,
    stdio: "pipe",
  });
}

function resolveDistPath(urlPathname: string): string {
  const pathname = decodeURIComponent(urlPathname);

  if (pathname === "/" || pathname === "") {
    return path.join(exampleDist, "index.html");
  }

  const normalizedPath = pathname.replace(/^\/+/, "");
  const directFile = path.join(exampleDist, normalizedPath);

  if (path.extname(normalizedPath)) {
    return directFile;
  }

  return path.join(exampleDist, normalizedPath, "index.html");
}

function contentTypeFor(filePath: string): string {
  const ext = path.extname(filePath).toLowerCase();
  switch (ext) {
    case ".html":
      return "text/html";
    case ".js":
      return "application/javascript";
    case ".css":
      return "text/css";
    case ".png":
      return "image/png";
    case ".svg":
      return "image/svg+xml";
    case ".json":
      return "application/json";
    default:
      return "application/octet-stream";
  }
}

async function fulfillDistRoute(route: Route, fontInjection: string) {
  const url = new URL(route.request().url());
  const filePath = resolveDistPath(url.pathname);

  try {
    if (filePath.endsWith(".html")) {
      const html = await readFile(filePath, "utf-8");
      await route.fulfill({
        contentType: contentTypeFor(filePath),
        body: html.replace("</head>", `${fontInjection}\n</head>`),
      });
      return;
    }

    const body = await readFile(filePath);
    await route.fulfill({
      contentType: contentTypeFor(filePath),
      body,
    });
  } catch {
    await route.fulfill({ status: 404, body: "not found" });
  }
}

async function routeBuiltExample(page: Page) {
  const fontInjection = await createFontInjection();

  await page.route("http://slides.test/**", async (route) => {
    await fulfillDistRoute(route, fontInjection);
  });
}

test.beforeAll(() => {
  buildSlidesExample();
});

test.describe("slides VRT", () => {
  test("build exports a deck pdf", async () => {
    const pdfPath = path.join(exampleDist, "slides", "deck.pdf");
    const file = await stat(pdfPath);
    expect(file.size).toBeGreaterThan(0);
  });

  test("renders the default slide shell", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "light" });
    await routeBuiltExample(page);

    await page.goto("http://slides.test/slides/1/");
    await page.locator(".ox-slide-page").waitFor();
    await page.waitForFunction(() => document.fonts.status === "loaded");

    await expect(page.locator(".ox-slide-page")).toHaveScreenshot("slides-default-shell.png", {
      animations: "disabled",
      caret: "hide",
      maxDiffPixelRatio: 0.02,
      scale: "css",
    });
  });

  test("renders the default slide shell in dark mode", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    await routeBuiltExample(page);

    await page.goto("http://slides.test/slides/1/");
    await page.locator(".ox-slide-page").waitFor();
    await page.waitForFunction(() => document.fonts.status === "loaded");

    await expect(page.locator(".ox-slide-page")).toHaveScreenshot("slides-default-shell-dark.png", {
      animations: "disabled",
      caret: "hide",
      maxDiffPixelRatio: 0.02,
      scale: "css",
    });
  });

  test("renders presenter mode with notes, timer, and next preview", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "light" });
    await routeBuiltExample(page);

    await page.goto("http://slides.test/slides/presenter/1/");
    await page.locator(".ox-presenter").waitFor();
    await page.waitForFunction(() => document.fonts.status === "loaded");
    await page.frameLocator('iframe[title="Current slide"]').locator(".ox-slide-page").waitFor();
    await page
      .frameLocator('iframe[title="Next slide preview"]')
      .locator(".ox-slide-page")
      .waitFor();

    await expect(page.locator(".ox-presenter")).toHaveScreenshot("slides-presenter-mode.png", {
      animations: "disabled",
      caret: "hide",
      maxDiffPixelRatio: 0.02,
      scale: "css",
    });
  });
});
