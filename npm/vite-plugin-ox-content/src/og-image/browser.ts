/**
 * Chromium browser session with automatic cleanup via Explicit Resource Management.
 *
 * Usage:
 *   await using session = await openBrowser();
 *   const png = await session.renderPage(html, 1200, 630);
 *   // browser.close() is called automatically when session goes out of scope
 */

import type { Browser, Page } from "playwright";
import { renderHtmlToPng, routePublicDir } from "./renderer";

const PLAYWRIGHT_BROWSER_INSTALL_HINT =
  "Install Playwright browsers with `npx playwright install chromium` to enable OG image generation.";

let chromiumUnavailableWarned = false;

/**
 * A browser session that can render HTML pages to PNG.
 * Implements AsyncDisposable for automatic cleanup via `await using`.
 */
export interface OgBrowserSession extends AsyncDisposable {
  renderPage(html: string, width: number, height: number, publicDir?: string): Promise<Buffer>;
}

async function launchChromium(): Promise<Browser> {
  const { chromium } = await import("playwright");
  return chromium.launch({
    headless: true,
    args: ["--no-sandbox", "--disable-setuid-sandbox", "--disable-dev-shm-usage", "--disable-gpu"],
  });
}

/** A set of independent sessions, closed together. */
export interface OgBrowserPool extends AsyncDisposable {
  sessions: OgBrowserSession[];
}

/**
 * Opens up to `size` independent browsers.
 *
 * Rendering does not parallelize inside one browser: every page shares a
 * single connection to it, so eight pages in one browser finish no sooner
 * than one does. Measured on 120 cards, milliseconds per image:
 *
 *     browsers   1     2     4     8
 *     pages 1    84.6  —     25.6  14.7
 *     pages 8    80.5  42.0  23.1  —
 *
 * Pages buy nothing; browsers scale nearly linearly. Each one costs its own
 * process, so this is bounded by the caller's `concurrency`, and
 * `concurrency: 1` keeps the old single-process footprint.
 *
 * Returns null when Chromium cannot be launched at all. A launch that fails
 * after the first still yields a working pool, one browser smaller.
 */
export async function openBrowserPool(size: number): Promise<OgBrowserPool | null> {
  const wanted = Math.max(1, size);
  const opened = await Promise.all(Array.from({ length: wanted }, () => openBrowser()));
  const sessions = opened.filter((session): session is OgBrowserSession => session !== null);

  if (sessions.length === 0) {
    return null;
  }

  return {
    sessions,
    async [Symbol.asyncDispose]() {
      await Promise.all(sessions.map((session) => session[Symbol.asyncDispose]()));
    },
  };
}

/**
 * Opens a Chromium browser and returns a session for rendering OG images.
 * Returns null if Playwright/Chromium is not available.
 *
 * The session implements AsyncDisposable — use `await using` for automatic cleanup:
 * ```ts
 * await using session = await openBrowser();
 * if (!session) return;
 * const png = await session.renderPage(html, 1200, 630);
 * ```
 */
export async function openBrowser(): Promise<OgBrowserSession | null> {
  try {
    const browser = await launchChromium();

    // One context for the session. `browser.newPage()` builds a fresh context
    // per call, so a site paid a context setup and teardown per image — which
    // is most of what an image costs and is why raising `concurrency` barely
    // moved the total.
    const context = await browser.newContext();

    // Pages are reused instead of closed: `setContent` replaces the document
    // outright, so there is nothing to carry over. The pool grows to whatever
    // the caller runs at once and no further.
    const idle: PooledPage[] = [];

    const acquire = async (): Promise<PooledPage> => {
      const pooled = idle.pop();
      return pooled ?? { page: await context.newPage() };
    };

    return {
      async renderPage(
        html: string,
        width: number,
        height: number,
        publicDir?: string,
      ): Promise<Buffer> {
        const pooled = await acquire();
        try {
          if (pooled.routedFor !== publicDir) {
            if (pooled.routedFor !== undefined) {
              await pooled.page.unrouteAll();
            }
            if (publicDir) {
              await routePublicDir(pooled.page, publicDir);
            }
            pooled.routedFor = publicDir;
          }
          const png = await renderHtmlToPng(pooled.page, html, width, height, publicDir);
          idle.push(pooled);
          return png;
        } catch (error) {
          // A page that threw may be mid-navigation or crashed; dropping it
          // keeps the failure from spreading to the next entry that would
          // have reused it.
          await pooled.page.close().catch(() => {});
          throw error;
        }
      },

      async [Symbol.asyncDispose]() {
        try {
          await browser.close();
        } catch {
          // Ignore close errors
        }
      },
    };
  } catch (err) {
    warnChromiumUnavailableOnce(err);
    return null;
  }
}

/** A pooled page and the public directory its route handler was set up for. */
interface PooledPage {
  page: Page;
  routedFor?: string;
}

function warnChromiumUnavailableOnce(err: unknown): void {
  if (chromiumUnavailableWarned) {
    return;
  }

  chromiumUnavailableWarned = true;
  console.warn(
    `[ox-content:og-image] Chromium not available, skipping OG image generation. ${formatChromiumUnavailableDetail(
      err,
    )}`,
  );
}

function formatChromiumUnavailableDetail(err: unknown): string {
  const message = err instanceof Error ? err.message : String(err);

  if (
    message.includes("Executable doesn't exist") ||
    message.includes("Please run the following command to download new browsers")
  ) {
    return PLAYWRIGHT_BROWSER_INSTALL_HINT;
  }

  return (
    message
      .split(/\r?\n/)
      .find((line) => line.trim())
      ?.trim() ?? "Unknown launch error."
  );
}
