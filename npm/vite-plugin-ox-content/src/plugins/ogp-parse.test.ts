import { describe, expect, it } from "vite-plus/test";
import { parseOgpFromHtml } from "./ogp/parse";

const PAGE = "https://example.com/posts/one";

function meta(...tags: string[]): string {
  return `<html><head>${tags.join("")}</head></html>`;
}

describe("Open Graph metadata parsing", () => {
  it("decodes character references in text metadata", () => {
    const data = parseOgpFromHtml(
      meta(
        `<meta property="og:title" content="Tips &amp; Tricks &#39;25">`,
        `<meta property="og:description" content="Caf&eacute; &lt;b&gt; &#x2014; done">`,
        `<meta property="og:site_name" content="A &amp; B">`,
      ),
      PAGE,
    );

    expect(data.title).toBe("Tips & Tricks '25");
    expect(data.description).toBe("Café <b> — done");
    expect(data.siteName).toBe("A & B");
  });

  it("decodes character references in the plain title tag", () => {
    const data = parseOgpFromHtml(meta(`<title>Ruby &amp; Rails</title>`), PAGE);
    expect(data.title).toBe("Ruby & Rails");
  });

  it("resolves a protocol-relative image against the page URL", () => {
    const data = parseOgpFromHtml(
      meta(`<meta property="og:image" content="//cdn.example.net/card.png">`),
      PAGE,
    );
    expect(data.image).toBe("https://cdn.example.net/card.png");
  });

  it("resolves a root-relative image against the page origin", () => {
    const data = parseOgpFromHtml(
      meta(`<meta property="og:image" content="/static/card.png">`),
      PAGE,
    );
    expect(data.image).toBe("https://example.com/static/card.png");
  });

  it("resolves a path-relative image against the page directory", () => {
    const data = parseOgpFromHtml(meta(`<meta property="og:image" content="card.png">`), PAGE);
    expect(data.image).toBe("https://example.com/posts/card.png");
  });

  it("decodes ampersands inside an image query string", () => {
    const data = parseOgpFromHtml(
      meta(`<meta property="og:image" content="https://cdn.example.net/c.png?w=1&amp;h=2">`),
      PAGE,
    );
    expect(data.image).toBe("https://cdn.example.net/c.png?w=1&h=2");
  });

  it("drops an image the fetcher would refuse to load", () => {
    for (const hostile of ["javascript:alert(1)", "http://127.0.0.1/card.png", "data:,x"]) {
      const data = parseOgpFromHtml(meta(`<meta property="og:image" content="${hostile}">`), PAGE);
      expect(data.image, hostile).toBeUndefined();
    }
  });

  it("takes the favicon from the page instead of a third-party lookup service", () => {
    const declared = parseOgpFromHtml(meta(`<link rel="icon" href="/assets/icon.svg">`), PAGE);
    expect(declared.favicon).toBe("https://example.com/assets/icon.svg");

    const fallback = parseOgpFromHtml(meta(`<title>No icon</title>`), PAGE);
    expect(fallback.favicon).toBe("https://example.com/favicon.ico");

    for (const data of [declared, fallback]) {
      expect(data.favicon).not.toContain("google.com");
    }
  });

  it("falls back to the host name when the page names no title", () => {
    const data = parseOgpFromHtml(meta(""), PAGE);
    expect(data.title).toBe("example.com");
  });
});
