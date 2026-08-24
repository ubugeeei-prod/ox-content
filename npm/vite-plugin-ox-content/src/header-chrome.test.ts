import { describe, expect, it } from "vite-plus/test";
import {
  parsePageChromeFlags,
  resolveHeaderNavItems,
  resolveLocaleLabel,
  resolvePageChromeOption,
} from "./header-chrome";
import { resolveSsgOptions } from "./ssg";
import { resolveTheme, themeToNapi } from "./theme";

describe("pageChrome Vite option", () => {
  it("Vite omitted => false; true/{} => enabled defaults", () => {
    expect(resolveSsgOptions(undefined).pageChrome).toBe(false);
    expect(resolveSsgOptions({}).pageChrome).toBe(false);
    expect(resolveSsgOptions({ pageChrome: false }).pageChrome).toBe(false);
    expect(resolveSsgOptions({ pageChrome: true }).pageChrome).toBe(true);
    expect(resolveSsgOptions({ pageChrome: {} }).pageChrome).toBe(true);
    expect(resolvePageChromeOption(undefined)).toBe(false);
    expect(resolvePageChromeOption(true)).toBe(true);
    expect(resolvePageChromeOption({})).toBe(true);
  });
});

describe("parsePageChromeFlags", () => {
  it("reads only boolean frontmatter flags", () => {
    expect(parsePageChromeFlags({})).toEqual({
      sidebar: undefined,
      outline: undefined,
      aside: undefined,
      footer: undefined,
      navbar: undefined,
      lastUpdated: undefined,
      editLink: undefined,
    });
    expect(
      parsePageChromeFlags({
        sidebar: false,
        outline: false,
        aside: false,
        footer: false,
        navbar: false,
        lastUpdated: false,
        editLink: false,
        title: "Guide",
      }),
    ).toEqual({
      sidebar: false,
      outline: false,
      aside: false,
      footer: false,
      navbar: false,
      lastUpdated: false,
      editLink: false,
    });
    expect(parsePageChromeFlags({ sidebar: "no", navbar: 0 })).toEqual({
      sidebar: undefined,
      outline: undefined,
      aside: undefined,
      footer: undefined,
      navbar: undefined,
      lastUpdated: undefined,
      editLink: undefined,
    });
  });
});

describe("locale-safe labels", () => {
  it("picks the current locale, language prefix, then configured fallback", () => {
    const text = { en: "Guide", ja: "ガイド", "en-GB": "Guide (UK)" };
    expect(resolveLocaleLabel(text, "ja")).toBe("ガイド");
    expect(resolveLocaleLabel(text, "en-GB")).toBe("Guide (UK)");
    expect(resolveLocaleLabel(text, "en-US")).toBe("Guide");
    expect(resolveLocaleLabel("Guide", "ja")).toBe("Guide");
    expect(resolveLocaleLabel({ ja: "ガイド", en: "Guide" }, "fr", "en")).toBe("Guide");
    expect(resolveLocaleLabel({ ja: "", en: "Guide" }, "ja", "en")).toBe("Guide");
    expect(resolveHeaderNavItems([{ text, link: "/guide/" }], "ja")).toEqual([
      { text: "ガイド", link: "/guide/" },
    ]);
  });

  it("ignores inherited locale-map properties and empty maps", () => {
    const inherited = Object.create({ ja: "inherited" }) as Record<string, string>;
    inherited.en = "Guide";
    expect(resolveLocaleLabel(inherited, "ja", "en")).toBe("Guide");
    expect(resolveLocaleLabel({}, "ja", "en")).toBe("");
  });
});

describe("theme header chrome", () => {
  it("passes header nav and announcement through", () => {
    const napi = themeToNapi(
      resolveTheme({
        nav: [{ text: "Guide", link: "/guide/" }],
        announcement: { text: "Hello", link: "https://example.com/news", dismissKey: "hi" },
      }),
    );
    expect(napi.nav).toEqual([{ text: "Guide", link: "/guide/" }]);
    expect(napi.announcement).toEqual({
      text: "Hello",
      link: "https://example.com/news",
      dismissKey: "hi",
    });
  });

  it("flattens locale map labels for the current page locale", () => {
    const napi = themeToNapi(
      resolveTheme({
        nav: [{ text: { en: "Guide", ja: "ガイド" }, link: "/guide/" }],
      }),
      "ja",
    );
    expect(napi.nav).toEqual([{ text: "ガイド", link: "/guide/" }]);
  });
});
