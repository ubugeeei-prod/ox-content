import * as path from "node:path";
import { describe, expect, it } from "vite-plus/test";
import {
  buildNavItems,
  buildThemeNavItems,
  formatTitle,
  getHref,
  getOutputPath,
  getPageLocale,
  getUrlPath,
  resolveSsgOptions,
} from "./ssg";

describe("resolveSsgOptions", () => {
  it("disables git timestamps by default", () => {
    expect(resolveSsgOptions(undefined).lastUpdated).toBe(false);
  });

  it("enables git timestamps when requested", () => {
    expect(resolveSsgOptions({ lastUpdated: true }).lastUpdated).toBe(true);
  });
});

describe("getPageLocale", () => {
  it("derives BCP 47 locales from localized paths", () => {
    const i18n = {
      enabled: true,
      dir: "content/i18n",
      defaultLocale: "en-US",
      locales: [
        { code: "en-US", name: "English" },
        { code: "zh-Hans-CN", name: "简体中文" },
      ],
      hideDefaultLocale: true,
      check: false,
      functionNames: ["t"],
    };

    expect(getPageLocale("zh-Hans-CN/guide", i18n)).toBe("zh-Hans-CN");
    expect(getPageLocale("guide", i18n)).toBe("en-US");
    expect(getPageLocale("guide", false)).toBeUndefined();
  });
});

describe("SSG route helpers", () => {
  it("resolves output paths, URL paths, and hrefs through NAPI", () => {
    const srcDir = path.join(process.cwd(), "docs");
    const outDir = path.join(process.cwd(), "dist");
    const inputPath = path.join(srcDir, "guide", "intro.md");

    expect(getOutputPath(inputPath, srcDir, outDir, ".html")).toBe(
      path.join(outDir, "guide", "intro", "index.html"),
    );
    expect(getUrlPath(inputPath, srcDir)).toBe("guide/intro");
    expect(getHref(inputPath, srcDir, "/docs/", ".html")).toBe("/docs/guide/intro/index.html");

    const mdxPath = path.join(srcDir, "components", "button.mdx");
    expect(getOutputPath(mdxPath, srcDir, outDir, ".html")).toBe(
      path.join(outDir, "components", "button", "index.html"),
    );
    expect(getUrlPath(mdxPath, srcDir)).toBe("components/button");
  });

  it("builds nav groups in the default SSG order", () => {
    const srcDir = path.join(process.cwd(), "docs");
    const files = [
      path.join(srcDir, "api", "reference.md"),
      path.join(srcDir, "examples", "basic.md"),
      path.join(srcDir, "index.md"),
      path.join(srcDir, "guide.md"),
    ];

    const groups = buildNavItems(files, srcDir, "/docs/", ".html");

    expect(groups.map((group) => group.title)).toEqual(["Guide", "Examples", "Api"]);
    expect(groups[0]?.items.map((item) => item.title)).toEqual(["Overview", "Guide"]);
  });

  it("builds theme sidebar nav groups safely", () => {
    const groups = buildThemeNavItems(
      [
        { text: "Intro", link: "/index.md" },
        {
          text: "Guide",
          collapsed: true,
          items: [{ text: "Install", link: "guide/install.md#cli" }],
        },
        { text: "Unsafe", link: "javascript:alert(1)" },
      ],
      "/docs/",
      ".html",
    );

    expect(groups[0]?.items[0]?.href).toBe("/docs/index.html");
    expect(groups[1]).toMatchObject({
      title: "Guide",
      collapsed: true,
      items: [
        { title: "Install", path: "guide/install", href: "/docs/guide/install/index.html#cli" },
      ],
    });
    expect(groups[2]?.items[0]?.href).toBe("#");
  });

  it("formats file names as titles through the Rust helper", () => {
    expect(formatTitle("getting_started-now")).toBe("Getting Started Now");
  });
});
