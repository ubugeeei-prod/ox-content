import { describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { generateOgImages, mapWithSessions, openBrowser, resolveOgImageOptions } from "./og-image";
import type { OgBrowserSession } from "./og-image";

/**
 * A stand-in for a browser session. Nothing here launches Chromium: what is
 * under test is how work is handed to sessions, not what a session does with
 * it.
 */
function fakeSessions(count: number): OgBrowserSession[] {
  return Array.from({ length: count }, () => ({
    renderPage: async () => Buffer.alloc(0),
    async [Symbol.asyncDispose]() {},
  }));
}

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("mapWithSessions", () => {
  it("returns results in input order however the sessions interleave", async () => {
    // The first item is the slowest, so a run that returned results in
    // completion order would put it last.
    const items = [0, 1, 2, 3, 4, 5, 6, 7];
    const results = await mapWithSessions(items, fakeSessions(4), async (item) => {
      for (let wait = item === 0 ? 20 : 0; wait > 0; wait--) {
        await tick();
      }
      return `item-${item}`;
    });

    expect(results).toEqual(items.map((item) => `item-${item}`));
  });

  it("renders every item exactly once", async () => {
    const items = Array.from({ length: 50 }, (_, index) => index);
    const seen: number[] = [];

    await mapWithSessions(items, fakeSessions(7), async (item) => {
      await tick();
      seen.push(item);
      return item;
    });

    expect(seen.sort((left, right) => left - right)).toEqual(items);
  });

  it("gives a session its next item without waiting for the others", async () => {
    // The batched version paid for its slowest member before starting the
    // next batch, so with one slow item and three sessions only two items
    // could be in flight. A shared cursor keeps all three busy.
    let inFlight = 0;
    let peak = 0;

    await mapWithSessions(
      Array.from({ length: 12 }, (_, index) => index),
      fakeSessions(3),
      async (item) => {
        inFlight++;
        peak = Math.max(peak, inFlight);
        for (let wait = item === 0 ? 30 : 1; wait > 0; wait--) {
          await tick();
        }
        inFlight--;
        return item;
      },
    );

    expect(peak).toBe(3);
  });

  it("uses one session without stranding work", async () => {
    const items = [1, 2, 3];
    expect(await mapWithSessions(items, fakeSessions(1), async (item) => item * 2)).toEqual([
      2, 4, 6,
    ]);
  });

  it("does nothing when there is nothing to render", async () => {
    expect(await mapWithSessions([], fakeSessions(4), async () => "x")).toEqual([]);
  });
});

/**
 * The Chromium path needs a browser, which the unit test job does not
 * install. Probing once keeps the file runnable everywhere: it covers the
 * real path where Chromium exists and skips where it does not.
 */
const chromiumSession = await openBrowser();
const hasChromium = chromiumSession !== null;
await chromiumSession?.[Symbol.asyncDispose]();

describe.skipIf(!hasChromium)("generateOgImages with Chromium", () => {
  it("writes every image the caller asked for", async () => {
    // Regression: the pool is disposed when `generateOgImages` returns, so
    // returning the render promise instead of awaiting it closed every
    // browser before the first render finished. Each page then failed with
    // "Target page, context or browser has been closed" — reported per page,
    // never thrown, so the build stayed green and produced nothing.
    const dir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-og-chromium-"));
    try {
      const pages = Array.from({ length: 6 }, (_, index) => ({
        outputPath: path.join(dir, `og-${index}.png`),
        props: { title: `Card ${index}`, description: "Rendered through Chromium" },
      }));

      const results = await generateOgImages(
        pages,
        resolveOgImageOptions({ cache: false, width: 300, height: 158 }),
        dir,
      );

      expect(results.map((result) => result.error)).toEqual(pages.map(() => undefined));
      expect(results.map((result) => result.outputPath)).toEqual(
        pages.map((page) => page.outputPath),
      );
      for (const page of pages) {
        const png = await fs.readFile(page.outputPath);
        expect(png.subarray(0, 8).toString("hex"), page.outputPath).toBe("89504e470d0a1a0a");
      }
    } finally {
      await fs.rm(dir, { recursive: true, force: true });
    }
  });
});
