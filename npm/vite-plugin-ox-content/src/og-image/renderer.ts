/**
 * HTML → PNG renderer using Chromium screenshots via Playwright.
 */

import * as path from "path";
import type { Page } from "playwright";

/**
 * Wraps template HTML in a minimal document with viewport locked to given dimensions.
 */
function wrapHtml(bodyHtml: string, width: number, height: number, useBaseUrl: boolean): string {
  const baseTag = useBaseUrl ? `\n<base href="http://localhost/">` : "";
  return `<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">${baseTag}
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
html, body { width: ${width}px; height: ${height}px; overflow: hidden; }
</style>
</head>
<body>${bodyHtml}</body>
</html>`;
}

const MIME_TYPES: Record<string, string> = {
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".webp": "image/webp",
  ".woff": "font/woff",
  ".woff2": "font/woff2",
  ".ttf": "font/ttf",
  ".css": "text/css",
  ".js": "application/javascript",
};

/**
 * Serves local assets from `publicDir` for anything the card requests.
 *
 * Registered once per page rather than once per render: a handler added on
 * every render stacks up on a page that is reused, and Playwright then walks
 * the whole stack for every request.
 */
export async function routePublicDir(page: Page, publicDir: string): Promise<void> {
  const fs = await import("fs/promises");
  await page.route("**/*", async (route) => {
    const url = new URL(route.request().url());
    // Only intercept paths that look like local assets (not data: or blob:)
    if (url.protocol !== "http:" && url.protocol !== "https:") {
      await route.continue();
      return;
    }
    const filePath = path.join(publicDir, url.pathname);
    try {
      const body = await fs.readFile(filePath);
      const ext = path.extname(filePath).toLowerCase();
      await route.fulfill({
        body,
        contentType: MIME_TYPES[ext] || "application/octet-stream",
      });
    } catch {
      await route.continue();
    }
  });
}

/**
 * Renders an HTML string to a PNG buffer using Chromium.
 *
 * @param page - Playwright page instance
 * @param html - HTML string from template function
 * @param width - Image width
 * @param height - Image height
 * @param publicDir - Optional public directory for serving local assets (images, fonts, etc.)
 * @returns PNG buffer
 */
export async function renderHtmlToPng(
  page: Page,
  html: string,
  width: number,
  height: number,
  publicDir?: string,
): Promise<Buffer> {
  await page.setViewportSize({ width, height });

  const fullHtml = wrapHtml(html, width, height, !!publicDir);
  // `load` fires once the document and its subresources — images, fonts, the
  // things a card actually shows — have loaded. `networkidle` additionally
  // waits out a 500ms quiet window, which a self-contained card spends idle,
  // and which every page pays whether or not it requests anything.
  await page.setContent(fullHtml, { waitUntil: "load" });

  const screenshot = await page.screenshot({
    type: "png",
    clip: { x: 0, y: 0, width, height },
  });

  return Buffer.from(screenshot);
}
