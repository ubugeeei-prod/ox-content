import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { resolveI18nOptions } from "./i18n";
import {
  localizeHeaderNavItems,
  localizeHref,
  localizeNavGroups,
  sitePathFromHref,
} from "./locale-nav";
import { buildSsg, buildThemeNavItems } from "./ssg";
import { resolveTheme } from "./theme";

const locales = [{ code: "en" }, { code: "ja" }, { code: "zh-Hans-CN" }];

const pages = [
  { path: "/", href: "/docs/index.html" },
  { path: "getting-started", href: "/docs/getting-started/index.html" },
  { path: "ja", href: "/docs/ja/index.html" },
  { path: "ja/getting-started", href: "/docs/ja/getting-started/index.html" },
  { path: "api/ssg", href: "/docs/api/ssg/index.html" },
];

function jaOptions() {
  return {
    locale: "ja",
    locales,
    defaultLocale: "en",
    hideDefaultLocale: true,
    pages,
    base: "/docs/",
  };
}

describe("sitePathFromHref", () => {
  it("strips base, index.html, markdown extensions, and trailing slashes", () => {
    expect(sitePathFromHref("/docs/getting-started/index.html", "/docs/")).toBe("getting-started");
    expect(sitePathFromHref("/docs/getting-started/", "/docs/")).toBe("getting-started");
    expect(sitePathFromHref("/docs/index.html", "/docs/")).toBe("");
    expect(sitePathFromHref("/getting-started/", "/")).toBe("getting-started");
    expect(sitePathFromHref("/getting-started.md", "/")).toBe("getting-started");
    expect(sitePathFromHref("/docs/getting-started.mdx", "/docs/")).toBe("getting-started");
  });

  it("rejects schemes and protocol-relative hrefs", () => {
    expect(sitePathFromHref("javascript:alert(1)", "/docs/")).toBeUndefined();
    expect(sitePathFromHref("https://example.com/x", "/docs/")).toBeUndefined();
    expect(sitePathFromHref("//evil.example/x", "/docs/")).toBeUndefined();
  });
});

describe("localizeNavGroups", () => {
  it("rewrites sidebar hrefs and paths to the current locale sibling", () => {
    const groups = localizeNavGroups(
      [
        {
          title: "Guide",
          items: [
            {
              title: "Getting Started",
              path: "getting-started",
              href: "/docs/getting-started/index.html",
              children: [],
            },
          ],
        },
      ],
      jaOptions(),
    );

    expect(groups[0]?.items[0]).toMatchObject({
      href: "/docs/ja/getting-started/index.html",
      path: "ja/getting-started",
    });
  });

  it("keeps the authored href when the sibling page is missing", () => {
    const groups = localizeNavGroups(
      [
        {
          title: "API",
          items: [
            {
              title: "SSG",
              path: "api/ssg",
              href: "/docs/api/ssg/index.html",
              children: [],
            },
          ],
        },
      ],
      jaOptions(),
    );

    expect(groups[0]?.items[0]).toMatchObject({
      href: "/docs/api/ssg/index.html",
      path: "api/ssg",
    });
  });

  it("leaves default-locale nav unchanged when the default locale is hidden", () => {
    const input = [
      {
        title: "Guide",
        items: [
          {
            title: "Getting Started",
            path: "getting-started",
            href: "/docs/getting-started/index.html",
            children: [],
          },
        ],
      },
    ];
    const groups = localizeNavGroups(input, { ...jaOptions(), locale: "en" });
    expect(groups).toBe(input);
  });

  it("preserves hash fragments on rewritten hrefs", () => {
    const groups = localizeNavGroups(
      [
        {
          title: "Guide",
          items: [
            {
              title: "Install",
              path: "getting-started",
              href: "/docs/getting-started/index.html#cli",
              children: [],
            },
          ],
        },
      ],
      jaOptions(),
    );
    expect(groups[0]?.items[0]?.href).toBe("/docs/ja/getting-started/index.html#cli");
  });

  it("does not rewrite javascript hrefs", () => {
    expect(localizeHref("javascript:alert(1)", jaOptions())).toBe("javascript:alert(1)");
  });

  it("rewrites nested sidebar children", () => {
    const groups = localizeNavGroups(
      [
        {
          title: "Guide",
          items: [
            {
              title: "Built-in",
              path: "built-in",
              href: "/docs/built-in/index.html",
              children: [
                {
                  title: "Cards",
                  path: "built-in/cards",
                  href: "/docs/built-in/cards/index.html",
                  children: [],
                },
              ],
            },
          ],
        },
      ],
      {
        ...jaOptions(),
        pages: [
          ...pages,
          { path: "built-in/cards", href: "/docs/built-in/cards/index.html" },
          { path: "ja/built-in/cards", href: "/docs/ja/built-in/cards/index.html" },
          { path: "zh-Hans-CN/built-in/cards", href: "/docs/zh-Hans-CN/built-in/cards/index.html" },
        ],
      },
    );
    expect(groups[0]?.items[0]?.children[0]).toMatchObject({
      href: "/docs/ja/built-in/cards/index.html",
      path: "ja/built-in/cards",
    });
  });

  it("rewrites resolved theme sidebar links that started as .md paths", () => {
    const groups = localizeNavGroups(
      buildThemeNavItems(
        [{ text: "Guide", items: [{ text: "Start", link: "/getting-started.md" }] }],
        "/docs/",
        ".html",
      ),
      jaOptions(),
    );
    expect(groups[0]?.items[0]).toMatchObject({
      href: "/docs/ja/getting-started/index.html",
      path: "ja/getting-started",
    });
  });
});

describe("localizeHeaderNavItems", () => {
  it("rewrites header links to the locale sibling", () => {
    const items = localizeHeaderNavItems(
      [{ text: { en: "Guide", ja: "ガイド" }, link: "/docs/getting-started/" }],
      jaOptions(),
    );
    expect(items?.[0]?.link).toBe("/docs/ja/getting-started/index.html");
    expect(items?.[0]?.text).toBe("ガイド");
  });
});

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("buildSsg locale nav", () => {
  it("keeps Japanese sidebar and header links on the ja sibling", async () => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-locale-nav-"));
    tempDirs.push(root);
    const srcDir = path.join(root, "content");
    await fs.mkdir(path.join(srcDir, "ja"), { recursive: true });
    await fs.writeFile(
      path.join(srcDir, "guide.md"),
      "---\ntitle: Guide\n---\n\n# Guide\n",
      "utf8",
    );
    await fs.writeFile(path.join(srcDir, "api.md"), "---\ntitle: API\n---\n\n# API\n", "utf8");
    await fs.writeFile(
      path.join(srcDir, "ja", "guide.md"),
      "---\ntitle: ガイド\n---\n\n# ガイド\n",
      "utf8",
    );

    const base = createDocsResolvedOptions();
    const i18n = resolveI18nOptions({
      enabled: true,
      defaultLocale: "en",
      locales: [
        { code: "en", name: "English" },
        { code: "ja", name: "日本語" },
      ],
      hideDefaultLocale: true,
      check: false,
    });
    expect(i18n).not.toBe(false);

    const result = await buildSsg(
      createDocsResolvedOptions({
        i18n,
        ssg: {
          ...base.ssg,
          localeSwitcher: true,
          theme: resolveTheme({
            sidebar: [
              {
                text: { en: "Guide", ja: "ガイド" },
                items: [
                  { text: { en: "Start", ja: "開始" }, link: "/guide.md" },
                  { text: { en: "API", ja: "API" }, link: "/api.md" },
                ],
              },
            ],
            nav: [{ text: "Guide", link: "/guide/" }],
          }),
        },
      }),
      root,
    );
    expect(result.errors).toEqual([]);
    expect(result.files.some((file) => file.includes(`${path.sep}ja${path.sep}guide`))).toBe(true);

    const jaGuide = await fs.readFile(path.join(root, "dist", "ja", "guide", "index.html"), "utf8");
    expect(jaGuide).toContain('href="/ja/guide/index.html"');
    expect(jaGuide).toContain('href="/api/index.html"');
    expect(jaGuide).toContain('<div class="nav-title">ガイド</div>');
    expect(jaGuide).toContain('class="nav-link active">開始</a>');
    expect(jaGuide).not.toContain("&lt;script");
    expect(jaGuide).not.toMatch(/aside[\s\S]*href="\/guide\/index\.html"/);
  });

  it("escapes hostile localized labels in nav and breadcrumbs", async () => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-locale-labels-"));
    tempDirs.push(root);
    const srcDir = path.join(root, "content");
    await fs.mkdir(path.join(srcDir, "ja"), { recursive: true });
    await fs.writeFile(path.join(srcDir, "guide.md"), "# Guide\n", "utf8");
    await fs.writeFile(path.join(srcDir, "ja", "guide.md"), "# ガイド\n", "utf8");
    const base = createDocsResolvedOptions();
    const i18n = resolveI18nOptions({
      enabled: true,
      defaultLocale: "en",
      locales: [
        { code: "en", name: "English" },
        { code: "ja", name: "日本語" },
      ],
      hideDefaultLocale: true,
      check: false,
    });
    expect(i18n).not.toBe(false);

    const result = await buildSsg(
      createDocsResolvedOptions({
        i18n,
        ssg: {
          ...base.ssg,
          breadcrumbs: true,
          theme: resolveTheme({
            breadcrumbs: true,
            sidebar: [
              {
                text: { en: "Guide", ja: '<img src=x onerror="alert(1)">' },
                items: [
                  { text: { en: "Start", ja: '<script>alert("x")</script>' }, link: "/guide.md" },
                ],
              },
            ],
          }),
        },
      }),
      root,
    );
    expect(result.errors).toEqual([]);
    const html = await fs.readFile(path.join(root, "dist", "ja", "guide", "index.html"), "utf8");
    expect(html).toContain("&lt;img src=x onerror=&quot;alert(1)&quot;&gt;");
    expect(html).toContain("&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;");
    expect(html).not.toContain('<img src=x onerror="alert(1)">');
    expect(html).not.toContain('<script>alert("x")</script>');
  });
});
