import { describe, expect, it } from "vite-plus/test";
import { localizeNavGroups } from "./locale-nav";
import { buildThemeNavItems } from "./ssg";

const pages = [
  { path: "getting-started", href: "/docs/getting-started/index.html" },
  { path: "ja/getting-started", href: "/docs/ja/getting-started/index.html" },
  { path: "built-in", href: "/docs/built-in/index.html" },
  { path: "ja/built-in", href: "/docs/ja/built-in/index.html" },
  { path: "cards", href: "/docs/cards/index.html" },
  { path: "ja/cards", href: "/docs/ja/cards/index.html" },
];

function localeOptions(locale = "ja") {
  return {
    locale,
    locales: [{ code: "en" }, { code: "ja" }],
    defaultLocale: "en",
    hideDefaultLocale: true,
    pages,
    base: "/docs/",
  };
}

describe("localized sidebar labels", () => {
  it("resolves top-level, linked-parent, and nested sidebar labels", () => {
    const groups = localizeNavGroups(
      buildThemeNavItems(
        [
          {
            text: { en: "Guide", ja: "ガイド" },
            collapsed: true,
            stickyCollapsed: true,
            items: [
              {
                text: { en: "Built-ins", ja: "組み込み" },
                link: "/built-in.md",
                items: [{ text: { en: "Cards", ja: "カード" }, link: "/cards.md" }],
              },
            ],
          },
        ],
        "/docs/",
        ".html",
      ),
      localeOptions(),
    );

    expect(groups[0]).toMatchObject({
      title: "ガイド",
      collapsed: true,
      stickyCollapsed: true,
      items: [
        {
          title: "組み込み",
          href: "/docs/ja/built-in/index.html",
          children: [{ title: "カード", href: "/docs/ja/cards/index.html" }],
        },
      ],
    });
  });

  it("falls back to the default locale before declaration order", () => {
    const groups = localizeNavGroups(
      buildThemeNavItems(
        [
          {
            text: { fr: "Guide français", en: "Guide" },
            items: [{ text: { fr: "Début", en: "Start" }, link: "/getting-started.md" }],
          },
        ],
        "/docs/",
        ".html",
      ),
      localeOptions(),
    );
    expect(groups[0]?.title).toBe("Guide");
    expect(groups[0]?.items[0]?.title).toBe("Start");
  });

  it("resolves default-locale labels when hrefs need no rewrite", () => {
    const groups = localizeNavGroups(
      buildThemeNavItems(
        [
          {
            text: { ja: "ガイド", en: "Guide" },
            items: [{ text: { ja: "開始", en: "Start" }, link: "/getting-started.md" }],
          },
        ],
        "/docs/",
        ".html",
      ),
      localeOptions("en"),
    );
    expect(groups[0]?.title).toBe("Guide");
    expect(groups[0]?.items[0]).toMatchObject({
      title: "Start",
      href: "/docs/getting-started/index.html",
    });
  });
});
