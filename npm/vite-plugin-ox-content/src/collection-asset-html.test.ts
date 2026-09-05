import { describe, expect, it } from "vite-plus/test";
import { rewriteCollectionAssetUrls } from ".";

describe("rewriteCollectionAssetUrls", () => {
  it("rewrites supported fragment references and leaves unsafe or unknown values alone", () => {
    const result = rewriteCollectionAssetUrls({
      html: [
        "<p>Fish & Chips ./chart.png</p>",
        '<img src="./chart.png?size=2#preview">',
        '<a href="/blog/example/%E5%9B%B3.png">encoded</a>',
        '<video poster="https://site.example/blog/example/chart.png"></video>',
        '<img src="./missing.png">',
        '<a href="https://other.example/blog/example/chart.png">external</a>',
        '<a href="#section">fragment</a>',
        '<img src="data:image/png;base64,abc">',
        '<a href="mailto:hello@example.com">mail</a>',
        '<a href="javascript:alert(1)">script-url</a>',
        '<a href="https://[broken">malformed</a>',
        '<script>const src = "./chart.png";</script>',
        '<style>.hero{background:url("./chart.png")}</style>',
      ].join(""),
      pagePath: "/blog/example/",
      origin: "https://site.example",
      manifest: manifest(),
    });

    expect(result.rewrites.map((rewrite) => [rewrite.attribute, rewrite.replacement])).toEqual([
      ["src", "/assets/content/chart.abc123.png?size=2#preview"],
      ["href", "/assets/content/chart.abc123.png"],
      ["poster", "/assets/content/chart.abc123.png"],
    ]);
    expect(result.html).toContain("Fish &#x26; Chips ./chart.png");
    expect(result.html).toContain('src="/assets/content/chart.abc123.png?size=2#preview"');
    expect(result.html).toContain('href="/assets/content/chart.abc123.png"');
    expect(result.html).toContain('poster="/assets/content/chart.abc123.png"');
    expect(result.html).toContain('src="./missing.png"');
    expect(result.html).toContain("https://other.example/blog/example/chart.png");
    expect(result.html).toContain('href="#section"');
    expect(result.html).toContain("data:image/png;base64,abc");
    expect(result.html).toContain("mailto:hello@example.com");
    expect(result.html).toContain("javascript:alert(1)");
    expect(result.html).toContain("https://[broken");
    expect(result.html).toContain('const src = "./chart.png";');
    expect(result.html).toContain('url("./chart.png")');
  });

  it("can parse full documents when requested", () => {
    const result = rewriteCollectionAssetUrls({
      document: true,
      html: [
        "<!doctype html>",
        '<html><head><link rel="preload" href="/blog/example/chart.png"></head>',
        '<body><img src="./chart.png"></body></html>',
      ].join(""),
      pagePath: "/blog/example/",
      manifest: manifest(),
    });

    expect(result.rewrites.map((rewrite) => rewrite.replacement)).toEqual([
      "/assets/content/chart.abc123.png",
      "/assets/content/chart.abc123.png",
    ]);
    expect(result.html).toContain("<!doctype html>");
    expect(result.html).toContain('<link rel="preload" href="/assets/content/chart.abc123.png">');
    expect(result.html).toContain('<img src="/assets/content/chart.abc123.png">');
  });
});

function manifest() {
  return {
    assets: [
      {
        sourcePath: "/repo/content/chart.png",
        publicPaths: ["/blog/example/chart.png", "/blog/example/%E5%9B%B3.png"],
        contentPath: "/assets/content/chart.abc123.png",
      },
    ],
  };
}
