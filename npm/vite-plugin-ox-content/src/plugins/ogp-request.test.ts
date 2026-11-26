import { afterEach, beforeEach, describe, expect, it } from "vite-plus/test";
import { clearOgpCache } from "./ogp/cache";
import { fetchOgpData } from "./ogp/fetch";
import { OGP_MAX_BYTES, installOgpLookup } from "./ogp/request";
import { isSafeOgpUrl } from "./ogp/url";

const originalFetch = globalThis.fetch;

beforeEach(() => {
  installOgpLookup(async () => ["93.184.215.14"]);
});

afterEach(() => {
  globalThis.fetch = originalFetch;
  installOgpLookup(undefined);
  clearOgpCache();
});

function card(title: string): string {
  return `<html><head><meta property="og:title" content="${title}"></head></html>`;
}

describe("Open Graph request targets", () => {
  it("rejects loopback and private hosts in every spelling", () => {
    for (const url of [
      "http://[::ffff:127.0.0.1]/admin",
      "http://[::ffff:169.254.169.254]/latest/meta-data/",
      "http://[::]/",
      "http://localhost./",
      "http://100.100.100.200/",
      "http://0x7f.1/",
      "http://user:pass@example.com/",
    ]) {
      expect(isSafeOgpUrl(url)).toBe(false);
    }
    expect(isSafeOgpUrl("https://example.com/post")).toBe(true);
    expect(isSafeOgpUrl("https://fcdomain.example/post")).toBe(true);
  });

  it("does not fetch a host that resolves to a private address", async () => {
    installOgpLookup(async () => ["10.0.0.5"]);
    const requested: string[] = [];
    globalThis.fetch = (async (input: string) => {
      requested.push(String(input));
      return new Response(card("Internal"));
    }) as typeof fetch;
    await expect(fetchOgpData("https://intranet.example/", { cache: false })).resolves.toBeNull();
    expect(requested).toEqual([]);
  });

  it("checks each redirect hop before following it", async () => {
    const requested: string[] = [];
    const modes: (RequestRedirect | undefined)[] = [];
    globalThis.fetch = (async (input: string, init?: RequestInit) => {
      requested.push(String(input));
      modes.push(init?.redirect);
      return new Response(null, {
        status: 302,
        headers: { location: "http://127.0.0.1:8080/admin" },
      });
    }) as typeof fetch;
    await expect(fetchOgpData("https://example.com/hop", { cache: false })).resolves.toBeNull();
    expect(requested).toEqual(["https://example.com/hop"]);
    expect(modes).toEqual(["manual"]);
  });

  it("follows a redirect to another public page", async () => {
    globalThis.fetch = (async (input: string) =>
      String(input) === "https://example.com/old"
        ? new Response(null, { status: 301, headers: { location: "/new" } })
        : new Response(card("Moved"))) as typeof fetch;
    await expect(
      fetchOgpData("https://example.com/old", { cache: false }),
    ).resolves.toMatchObject({ title: "Moved" });
  });

  it("stops reading an endless body at the size limit", async () => {
    const chunk = new TextEncoder().encode(`${card("Endless")}${" ".repeat(64 * 1024)}`);
    let pulled = 0;
    globalThis.fetch = (async () =>
      new Response(
        new ReadableStream<Uint8Array>({
          pull(controller) {
            pulled += chunk.byteLength;
            controller.enqueue(chunk);
          },
        }),
      )) as typeof fetch;
    await expect(
      fetchOgpData("https://example.com/endless", { cache: false }),
    ).resolves.toMatchObject({ title: "Endless" });
    expect(pulled).toBeLessThan(OGP_MAX_BYTES * 2);
  });

  it("applies the timeout while the body is still streaming", async () => {
    globalThis.fetch = (async (_input: string, init?: RequestInit) =>
      new Response(
        new ReadableStream<Uint8Array>({
          start(controller) {
            controller.enqueue(new TextEncoder().encode("<html><head>"));
            init?.signal?.addEventListener("abort", () => controller.error(init.signal?.reason));
          },
        }),
      )) as typeof fetch;
    await expect(
      fetchOgpData("https://example.com/stalled", { cache: false, timeout: 50 }),
    ).resolves.toBeNull();
  });
});
