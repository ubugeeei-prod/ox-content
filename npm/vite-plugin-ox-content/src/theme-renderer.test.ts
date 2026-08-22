import { describe, expect, it } from "vite-plus/test";
import { createTheme, renderPage, type PageData } from "./theme-renderer";
import { usePageProps, useSiteConfig } from "./page-context";
import { jsx, raw } from "./jsx-html";

const page: PageData = {
  title: "Guide",
  description: "How it works",
  html: "<p>body</p>",
  toc: [],
  path: "content/guide.md",
  url: "/guide/index.html",
  frontmatter: {},
};

describe("renderPage", () => {
  it("lets the component own the whole document", () => {
    const Theme = ({ children }: { children: { __html: string } }) => {
      const current = usePageProps();
      const site = useSiteConfig();
      return jsx("html", {
        lang: "ja",
        children: [
          jsx("head", { children: jsx("title", { children: `${current.title} | ${site.name}` }) }),
          jsx("body", { children }),
        ],
      });
    };

    const html = renderPage(page, {
      theme: Theme,
      siteName: "Docs",
      base: "/",
      nav: [],
      pages: [page],
    });

    expect(html.startsWith("<!DOCTYPE html>")).toBe(true);
    expect(html).toContain('<html lang="ja">');
    expect(html).toContain("<title>Guide | Docs</title>");
    expect(html).toContain("<p>body</p>");
    // No theme stylesheet, no runtime scripts — the component decides.
    expect(html).not.toContain("ox-content-core.js");
    expect(html).not.toContain("--octc-");
  });

  it("picks the layout named by the page's frontmatter", () => {
    const theme = createTheme({
      layouts: {
        default: ({ children }) => jsx("main", { children }),
        entry: ({ children }) => jsx("section", { class: "entry", children }),
      },
    });

    const entryPage: PageData = { ...page, layout: "entry" };
    const html = renderPage(entryPage, {
      theme,
      siteName: "Docs",
      base: "/",
      nav: [],
      pages: [entryPage],
    });

    expect(html).toContain('<section class="entry">');
    expect(html).not.toContain("<main>");
  });
});

describe("raw", () => {
  it("passes page html through without escaping", () => {
    expect(raw("<em>x</em>").__html).toBe("<em>x</em>");
  });
});
