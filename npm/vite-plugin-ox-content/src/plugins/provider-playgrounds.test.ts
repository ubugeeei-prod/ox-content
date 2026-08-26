import { afterEach, describe, expect, it } from "vite-plus/test";
import {
  clearProviderPlaygroundCache,
  enrichProviderPlaygroundEmbeds,
  parsePlaygroundReference,
  type ProviderPlaygroundFetch,
} from "./provider-playgrounds";

const originalWarn = console.warn;

afterEach(() => {
  console.warn = originalWarn;
  clearProviderPlaygroundCache();
});

describe("playground provider cards", () => {
  it("parses supported playground URLs", () => {
    expect(parsePlaygroundReference("CodePen", "https://codepen.io/ubugeeei/pen/abc123")).toEqual({
      provider: "codepen",
      canonicalUrl: "https://codepen.io/ubugeeei/pen/abc123",
      title: "abc123",
      author: "ubugeeei",
      apiUrl:
        "https://codepen.io/api/oembed?format=json&url=https%3A%2F%2Fcodepen.io%2Fubugeeei%2Fpen%2Fabc123",
      embedUrl: "https://codepen.io/ubugeeei/embed/abc123?default-tab=result",
    });
    expect(
      parsePlaygroundReference("JSFiddle", "https://jsfiddle.net/ubugeeei/abc123/2/"),
    ).toMatchObject({
      provider: "jsfiddle",
      canonicalUrl: "https://jsfiddle.net/ubugeeei/abc123/2/",
      title: "abc123",
      author: "ubugeeei",
    });
    expect(
      parsePlaygroundReference("Observable", "https://observablehq.com/@d3/bar-chart"),
    ).toMatchObject({
      provider: "observable",
      canonicalUrl: "https://observablehq.com/@d3/bar-chart",
      title: "bar chart",
      author: "@d3",
    });
  });

  it("rejects unsafe or unsupported playground URLs", () => {
    expect(parsePlaygroundReference("CodePen", "http://codepen.io/ubugeeei/pen/abc123")).toBeNull();
    expect(
      parsePlaygroundReference("JSFiddle", "https://user:pass@jsfiddle.net/ubugeeei/abc123"),
    ).toBeNull();
    expect(parsePlaygroundReference("Observable", "https://observablehq.com/docs")).toBeNull();
    expect(parsePlaygroundReference("CodePen", "https://codepen.io.evil/u/pen/abc")).toBeNull();
  });

  it("enriches CodePen oEmbed metadata and caches by reference", async () => {
    const requests: string[] = [];
    const fetchImpl: ProviderPlaygroundFetch = async (input) => {
      const url = requestUrl(input);
      requests.push(url);
      return okJson({
        title: "Fetched Pen",
        author_name: "Pen Author",
        thumbnail_url: "https://shots.codepen.io/pen.png",
      });
    };
    const input = [
      '<CodePen url="https://codepen.io/ubugeeei/pen/abc123"></CodePen>',
      '<JSFiddle url="https://jsfiddle.net/ubugeeei/abc123/2/"></JSFiddle>',
      '<Observable url="https://observablehq.com/@d3/bar-chart"></Observable>',
    ].join("\n");

    const html = await enrichProviderPlaygroundEmbeds(input, {}, fetchImpl);
    expect(html).toContain('title="Fetched Pen"');
    expect(html).toContain('author="Pen Author"');
    expect(html).toContain('image="https://shots.codepen.io/pen.png"');
    expect(html).toContain(
      '<JSFiddle url="https://jsfiddle.net/ubugeeei/abc123/2/" title="abc123"',
    );
    expect(html).toContain(
      '<Observable url="https://observablehq.com/@d3/bar-chart" title="bar chart"',
    );

    await enrichProviderPlaygroundEmbeds(input, {}, fetchImpl);
    expect(requests).toEqual([
      "https://codepen.io/api/oembed?format=json&url=https%3A%2F%2Fcodepen.io%2Fubugeeei%2Fpen%2Fabc123",
    ]);
  });

  it("adds iframe URLs only when explicitly enabled", async () => {
    const html = await enrichProviderPlaygroundEmbeds(
      '<CodePen url="https://codepen.io/ubugeeei/pen/abc123"></CodePen>',
      { fetch: false, iframe: true },
    );

    expect(html).toContain('embed="https://codepen.io/ubugeeei/embed/abc123?default-tab=result"');
  });

  it("keeps deterministic card attributes when metadata is unavailable", async () => {
    const warnings: string[] = [];
    console.warn = (message?: unknown) => {
      warnings.push(String(message));
    };
    const html = await enrichProviderPlaygroundEmbeds(
      '<CodePen url="https://codepen.io/ubugeeei/pen/private"></CodePen>',
      {},
      async () => new Response("{}", { status: 429 }),
    );

    expect(html).toContain('title="private"');
    expect(html).toContain('author="ubugeeei"');
    expect(warnings[0]).toContain("429");
    expect(warnings[0]).toContain("link-only playground card");
  });
});

function requestUrl(input: Parameters<typeof fetch>[0]): string {
  if (typeof input === "string") return input;
  if (input instanceof URL) return input.href;
  return input.url;
}

function okJson(value: unknown): Response {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { "content-type": "application/json" },
  });
}
