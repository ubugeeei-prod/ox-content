import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vite-plus/test";

const require = createRequire(new URL("./apps/astro/package.json", import.meta.url));
const entry = join(dirname(require.resolve("astro/package.json")), "dist/assets/build/remote.js");
const { loadRemoteImage, revalidateRemoteImage } = await import(pathToFileURL(entry).href);
const source = "https://images.example.test/photo.png";

describe("benchmark-only Astro remote cache replacement", () => {
  it("preserves bytes and validators without caching a public or private response", async () => {
    for (const cacheControl of ["public, max-age=86400", "private, max-age=86400"]) {
      const result = await loadRemoteImage(
        source,
        async () =>
          new Response("image bytes", {
            headers: {
              "cache-control": cacheControl,
              etag: '"v1"',
              "set-cookie": "session=secret",
            },
          }),
      );
      expect(result.data.toString()).toBe("image bytes");
      expect(result.etag).toBe('"v1"');
      expect(result.expires).toBeLessThanOrEqual(Date.now());
    }
  });

  it("keeps conditional revalidation and previous validators after a 304", async () => {
    const previous = { etag: '"v1"', lastModified: "Wed, 01 Oct 2025 00:00:00 GMT" };
    const result = await revalidateRemoteImage(source, previous, async (request: Request) => {
      expect(request.headers.get("if-none-match")).toBe(previous.etag);
      expect(request.headers.get("if-modified-since")).toBe(previous.lastModified);
      return new Response(null, {
        status: 304,
        headers: { "cache-control": "public, max-age=86400" },
      });
    });
    expect(result.data).toBeNull();
    expect(result.etag).toBe(previous.etag);
    expect(result.lastModified).toBe(previous.lastModified);
    expect(result.expires).toBeLessThanOrEqual(Date.now());
  });

  it("rejects failures and disallowed redirects", async () => {
    await expect(
      loadRemoteImage(source, async () => new Response(null, { status: 500 })),
    ).rejects.toThrow("500");
    await expect(
      loadRemoteImage(
        source,
        async () =>
          new Response(null, {
            status: 302,
            headers: { location: "https://elsewhere.example.test/image.png" },
          }),
      ),
    ).rejects.toThrow("not an allowed remote location");
  });
});
