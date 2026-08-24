import { describe, expect, it } from "vite-plus/test";
import {
  buildLocalePaths,
  pathForLocale,
  remainderPath,
  resolveLocaleSwitcherOption,
} from "./locale-switcher";

describe("resolveLocaleSwitcherOption", () => {
  it("treats omitted as false", () => {
    expect(resolveLocaleSwitcherOption(undefined)).toBe(false);
  });

  it("treats true as true", () => {
    expect(resolveLocaleSwitcherOption(true)).toBe(true);
  });

  it("treats an empty object as true", () => {
    expect(resolveLocaleSwitcherOption({})).toBe(true);
  });
});

describe("buildLocalePaths", () => {
  const locales = [
    { code: "en", name: "English" },
    { code: "ja", name: "日本語" },
  ];

  it("uses the sibling href when that locale page exists", () => {
    const paths = buildLocalePaths({
      currentPath: "ja/guide",
      locales,
      defaultLocale: "en",
      hideDefaultLocale: true,
      pages: [
        { path: "guide", href: "/docs/guide/index.html" },
        { path: "ja/guide", href: "/docs/ja/guide/index.html" },
      ],
      base: "/docs/",
    });

    expect(paths).toEqual([
      { code: "en", href: "/docs/guide/index.html", root: "/docs/" },
      { code: "ja", href: "/docs/ja/guide/index.html", root: "/docs/ja/" },
    ]);
  });

  it("omits href when the sibling is missing", () => {
    const paths = buildLocalePaths({
      currentPath: "guide",
      locales,
      defaultLocale: "en",
      hideDefaultLocale: true,
      pages: [{ path: "guide", href: "/docs/guide/index.html" }],
      base: "/docs/",
    });

    expect(paths).toEqual([
      { code: "en", href: "/docs/guide/index.html", root: "/docs/" },
      { code: "ja", href: undefined, root: "/docs/ja/" },
    ]);
  });

  it("prefixes every locale when hideDefaultLocale is false", () => {
    expect(remainderPath("en/guide", "en")).toBe("guide");
    expect(pathForLocale("guide", "en", "en", false)).toBe("en/guide");
    expect(pathForLocale("guide", "ja", "en", false)).toBe("ja/guide");
  });
});
