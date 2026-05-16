import { describe, expect, it } from "vite-plus/test";
import { resolveNavigationGroups } from "./ssg";
import {
  convertVitePressNav,
  convertVitePressSidebar,
  fromVitePressConfig,
  generateVitePressMigrationConfig,
  normalizeVitePressFrontmatter,
} from "./vitepress";

describe("vitepress migration helpers", () => {
  it("converts a flat VitePress sidebar into ox-content navigation groups", () => {
    const navigation = convertVitePressSidebar([
      { text: "Home", link: "/" },
      { text: "Getting Started", link: "/getting-started" },
      { text: "API", link: "/api.md" },
    ]);

    expect(navigation).toEqual([
      {
        title: "Guide",
        items: [
          { title: "Home", path: "/" },
          { title: "Getting Started", path: "/getting-started" },
          { title: "API", path: "/api" },
        ],
      },
    ]);
  });

  it("converts VitePress nav dropdowns into grouped sidebar navigation", () => {
    const navigation = convertVitePressNav([
      { text: "Guide", items: [{ text: "Intro", link: "/guide/intro" }] },
      { text: "GitHub", link: "https://github.com/ubugeeei/ox-content" },
    ]);

    expect(navigation).toEqual([
      {
        title: "Navigation",
        items: [
          {
            title: "GitHub",
            href: "https://github.com/ubugeeei/ox-content",
          },
        ],
      },
      {
        title: "Guide",
        items: [{ title: "Intro", path: "/guide/intro" }],
      },
    ]);
  });

  it("maps VitePress config into ox-content options and preserves user overrides", () => {
    const options = fromVitePressConfig(
      {
        title: "Docs",
        base: "/docs/",
        themeConfig: {
          logo: "/logo.svg",
          socialLinks: [{ icon: "github", link: "https://github.com/ubugeeei/ox-content" }],
          footer: { copyright: "2026" },
          sidebar: [{ text: "Intro", link: "/intro" }],
          search: { placeholder: "Search VitePress docs" },
        },
      },
      {
        srcDir: "docs",
        ssg: {
          theme: {
            footer: {
              message: "Migrated from VitePress",
            },
          },
        },
      },
    );

    expect(options.base).toBe("/docs/");
    expect(options.srcDir).toBe("docs");
    expect(options.search).toEqual({ placeholder: "Search VitePress docs" });

    expect(options.ssg).not.toBe(false);
    if (options.ssg === false) {
      throw new Error("Expected migrated SSG options");
    }

    expect(options.ssg.siteName).toBe("Docs");
    expect(options.ssg.navigation).toEqual([
      {
        title: "Guide",
        items: [{ title: "Intro", path: "/intro" }],
      },
    ]);
    expect(options.ssg.theme?.header?.logo).toBe("/logo.svg");
    expect(options.ssg.theme?.socialLinks?.github).toBe("https://github.com/ubugeeei/ox-content");
    expect(options.ssg.theme?.footer?.copyright).toBe("2026");
    expect(options.ssg.theme?.footer?.message).toBe("Migrated from VitePress");
  });

  it("generates an editable ox-content options module", () => {
    const source = generateVitePressMigrationConfig(
      {
        title: "Docs",
        base: "/docs/",
        themeConfig: {
          sidebar: [{ text: "Intro", link: "/intro.md" }],
          search: { placeholder: "Search docs" },
        },
      },
      {
        srcDir: "docs",
        outDir: "dist",
      },
    );

    expect(source).toContain('import type { OxContentOptions } from "@ox-content/vite-plugin";');
    expect(source).toContain("satisfies OxContentOptions");
    expect(source).toContain('base: "/docs/"');
    expect(source).toContain('srcDir: "docs"');
    expect(source).toContain('outDir: "dist"');
    expect(source).toContain('path: "/intro"');
  });

  it("normalizes VitePress home frontmatter into ox-content entry frontmatter", () => {
    const frontmatter = normalizeVitePressFrontmatter({
      layout: "home",
      hero: {
        name: "Docs",
        image: {
          light: "/logo-light.svg",
          dark: "/logo-dark.svg",
          width: "120",
          height: 80,
        },
      },
    });

    expect(frontmatter).toEqual({
      layout: "entry",
      hero: {
        name: "Docs",
        image: {
          src: "/logo-light.svg",
          width: 120,
          height: 80,
        },
      },
    });
  });

  it("derives HTML hrefs from migrated navigation paths", () => {
    const navigation = resolveNavigationGroups(
      [
        {
          title: "Guide",
          items: [
            { title: "Intro", path: "/intro" },
            { title: "Top", path: "/guide", href: "/guide#top" },
          ],
        },
      ],
      "/docs/",
      ".html",
    );

    expect(navigation).toEqual([
      {
        title: "Guide",
        items: [
          {
            title: "Intro",
            path: "/intro",
            href: "/docs/intro/index.html",
          },
          {
            title: "Top",
            path: "/guide",
            href: "/docs/guide/index.html#top",
          },
        ],
      },
    ]);
  });
});
