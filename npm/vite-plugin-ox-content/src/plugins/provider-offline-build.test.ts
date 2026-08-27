import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vite-plus/test";
import {
  clearProviderPackageCache,
  enrichProviderPackageEmbeds,
  type ProviderPackageFetch,
} from "./provider-packages";
import { resetEmbedFailureReporting } from "./provider-failure";

/**
 * The point of the offline case: a build with a warm disk cache must not care
 * that the network is gone. Same output, and nothing to say about it.
 */
const INPUT = '<NpmPackage url="https://www.npmjs.com/package/vite"></NpmPackage>';

const PAYLOAD = {
  name: "vite",
  "dist-tags": { latest: "7.0.0" },
  versions: { "7.0.0": { version: "7.0.0", license: "MIT", description: "Next generation" } },
  time: { "7.0.0": "2026-08-26T00:00:00Z" },
};

function respondingFetch(): { fetch: ProviderPackageFetch; calls: string[] } {
  const calls: string[] = [];
  const fetch: ProviderPackageFetch = async (url) => {
    calls.push(String(url));
    return new Response(JSON.stringify(PAYLOAD), { status: 200 });
  };
  return { fetch, calls };
}

/** Fails the way Node does when there is no network at all. */
function offlineFetch(): { fetch: ProviderPackageFetch; calls: string[] } {
  const calls: string[] = [];
  const fetch: ProviderPackageFetch = async (url) => {
    calls.push(String(url));
    throw Object.assign(new Error("fetch failed"), { cause: { code: "ENOTFOUND" } });
  };
  return { fetch, calls };
}

describe("an offline build", () => {
  let cacheDir: string;
  let warned: string[];

  beforeEach(async () => {
    cacheDir = await mkdtemp(path.join(tmpdir(), "ox-offline-"));
    resetEmbedFailureReporting();
    warned = [];
    vi.spyOn(console, "warn").mockImplementation((message: unknown) => {
      warned.push(String(message));
    });
  });

  afterEach(async () => {
    vi.restoreAllMocks();
    clearProviderPackageCache();
    await rm(cacheDir, { recursive: true, force: true });
  });

  it("renders what a warm cache holds, without reaching for the network", async () => {
    const options = { persistCache: true, cacheDir };

    const online = respondingFetch();
    const warm = await enrichProviderPackageEmbeds(INPUT, options, online.fetch);
    expect(online.calls).toHaveLength(1);

    // A fresh process: only the disk cache survives.
    clearProviderPackageCache();
    resetEmbedFailureReporting();
    warned = [];

    const offline = offlineFetch();
    const cold = await enrichProviderPackageEmbeds(INPUT, options, offline.fetch);

    expect(cold).toBe(warm);
    expect(offline.calls).toEqual([]);
    expect(warned).toEqual([]);
  });

  it("says the network is gone once, however many embeds miss the cache", async () => {
    const input = Array.from(
      { length: 5 },
      (_, index) => `<NpmPackage url="https://www.npmjs.com/package/pkg-${index}"></NpmPackage>`,
    ).join("\n");

    const offline = offlineFetch();
    await enrichProviderPackageEmbeds(input, { persistCache: true, cacheDir }, offline.fetch);

    expect(offline.calls).toHaveLength(5);
    expect(warned).toHaveLength(1);
    expect(warned[0]).toContain("No network");
  });
});
