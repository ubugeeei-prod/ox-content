import { describe, expect, it } from "vite-plus/test";
import { renderViewerHtml } from "./og-viewer";
import type { ResolvedOptions } from "./types";

const options = {
  ogImage: false,
  ssg: { generateOgImage: false, siteUrl: "https://docs.example/</script><b>" },
} as unknown as ResolvedOptions;

const hostile = "Hi</script><script>window.__ran=1</script> ";

function page(overrides: Record<string, unknown> = {}) {
  return {
    path: "guide.md",
    urlPath: "guide",
    title: hostile,
    description: `<!--${hostile}`,
    author: hostile,
    tags: [hostile],
    ogImageUrl: '/x" data-x="1',
    warnings: [],
    ...overrides,
  };
}

describe("OG viewer page", () => {
  it("keeps frontmatter text inside the inline script data", () => {
    const pages = [page()];
    const html = renderViewerHtml(pages, options);

    expect(html.match(/<script\b/g)).toHaveLength(1);
    expect(html.match(/<\/script>/g)).toHaveLength(1);
    const data = /let pages = (.*);\n/.exec(html)?.[1] ?? "";
    expect(JSON.parse(data)).toEqual(pages);
  });

  it("escapes quotes in values placed inside attributes", () => {
    const html = renderViewerHtml([page()], options);
    const source = /function esc\(s\) \{[\s\S]*?\n {4}\}/.exec(html)?.[0] ?? "";
    const esc = new Function(`${source}; return esc;`)() as (value: unknown) => string;

    expect(esc('/x" data-x="1')).toBe("/x&quot; data-x=&quot;1");
    expect(esc("<b>'&'</b>")).toBe("&lt;b&gt;&#39;&amp;&#39;&lt;/b&gt;");
  });
});
