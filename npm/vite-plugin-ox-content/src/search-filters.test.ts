import { describe, expect, it } from "vite-plus/test";
import {
  injectSearchLocaleFilters,
  injectSearchVersionFilters,
  searchDocumentLocale,
} from "./search-filters";

const dialog = `<div class="search-modal"><div class="search-results"></div></div>`;

describe("searchDocumentLocale", () => {
  it("uses the first locale segment and otherwise the default locale", () => {
    expect(searchDocumentLocale("/ja/built-in/search", ["en", "ja"], "en")).toBe("ja");
    expect(searchDocumentLocale("/built-in/search", ["en", "ja"], "en")).toBe("en");
    expect(searchDocumentLocale("ja/guide", ["en", "ja"], "en")).toBe("ja");
  });

  it("strips version prefixes before reading the locale segment", () => {
    expect(searchDocumentLocale("/2.90/ja/guide", ["en", "ja"], "en", ["2.90"])).toBe("ja");
    expect(searchDocumentLocale("/2.90/guide", ["en", "ja"], "en", ["2.90"])).toBe("en");
    expect(searchDocumentLocale("/docs/2.90/ja/guide", ["en", "ja"], "en", ["docs/2.90"])).toBe(
      "ja",
    );
  });
});

describe("injectSearchLocaleFilters", () => {
  it("leaves single-locale sites unchanged", () => {
    expect(
      injectSearchLocaleFilters(dialog, {
        locales: [{ code: "en", name: "English" }],
        current: "en",
        defaultLocale: "en",
      }),
    ).toBe(dialog);
  });

  it("defaults to the current page locale and keeps an All languages option", () => {
    const html = injectSearchLocaleFilters(dialog, {
      locales: [
        { code: "en", name: "English" },
        { code: "ja", name: "日本語" },
      ],
      current: "ja",
      defaultLocale: "en",
    });
    expect(html).toContain('data-search-filter="locale"');
    expect(html).toContain('data-default-locale="en"');
    expect(html).toContain('<option value="">All languages</option>');
    expect(html).toContain('<option value="ja" selected>日本語</option>');
    expect(html).toContain('<option value="en">English</option>');
    expect(html).toContain('data-search-filter-label="locale"');
    expect(html).not.toContain('data-search-filter-label="locale" hidden');
  });

  it("escapes locale labels", () => {
    const html = injectSearchLocaleFilters(dialog, {
      locales: [
        { code: "en", name: "English" },
        { code: "ja", name: `<img src=x onerror=alert(1)>` },
      ],
      current: "en",
      defaultLocale: "en",
    });
    expect(html).toContain("&lt;img src=x onerror=alert(1)&gt;");
    expect(html).not.toContain("<img src=x");
  });
});

describe("injectSearchVersionFilters", () => {
  it("writes keyboard-accessible version options with index URLs", () => {
    const html = injectSearchVersionFilters(dialog, [
      {
        id: "3.0.0-alpha",
        label: "3.0.0-alpha",
        prefix: "",
        indexUrl: "/search-index.json",
        current: true,
      },
      {
        id: "2.90.0",
        label: "2.90.0",
        prefix: "2.90",
        indexUrl: "/2.90/search-index.json",
        current: false,
      },
    ]);
    expect(html).toContain('data-search-filter="version"');
    expect(html).toContain('aria-label="Version"');
    expect(html).toContain('data-index="/2.90/search-index.json"');
    expect(html).toContain('data-prefix="2.90"');
    expect(html).toContain('<option value="3.0.0-alpha"');
    expect(html).toContain(" selected>");
    expect(html).not.toContain('data-search-filter-label="version" hidden');
  });

  it("rejects javascript index URLs", () => {
    const html = injectSearchVersionFilters(dialog, [
      {
        id: "live",
        label: "Live",
        prefix: "",
        indexUrl: "/search-index.json",
        current: true,
      },
      {
        id: "evil",
        label: "Evil",
        prefix: "x",
        indexUrl: "javascript:alert(1)",
        current: false,
      },
    ]);
    expect(html).toBe(dialog);
  });
});
