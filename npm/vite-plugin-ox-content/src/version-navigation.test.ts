import { describe, expect, it } from "vite-plus/test";
import { localizeNavGroups } from "./locale-nav";
import {
  createVersionNavigationContext,
  rewriteVersionedHeaderNavItems,
  rewriteVersionedNavGroups,
  unversionedPath,
  versionedLocaleRoots,
} from "./version-navigation";

function context(base = "/docs/") {
  return createVersionNavigationContext({
    prefix: "2.90",
    base,
    pages: [
      { path: "", versionedPath: "2.90", href: `${base}2.90/`, sourcePath: "" },
      {
        path: "getting-started",
        versionedPath: "2.90/getting-started",
        href: `${base}2.90/getting-started/`,
        sourcePath: "guide/intro",
        aliases: ["/start"],
      },
      {
        path: "ja/getting-started",
        versionedPath: "2.90/ja/getting-started",
        href: `${base}2.90/ja/getting-started/`,
      },
    ],
    redirects: { "/legacy": "/getting-started", "/older": "/legacy" },
  });
}

describe("version navigation", () => {
  it("rewrites canonical, permalink-source, alias, nested, and header links", () => {
    const ctx = context();
    const groups = rewriteVersionedNavGroups(
      [
        {
          title: "Guide",
          items: [
            {
              title: "Start",
              path: "guide/intro",
              href: "/docs/guide/intro/",
              children: [{ title: "Alias", path: "start", href: "/start#install" }],
            },
            { title: "Redirect alias", path: "legacy", href: "/legacy" },
            { title: "Chained alias", path: "older", href: "/older" },
          ],
        },
      ],
      ctx,
    );

    expect(groups[0]?.items[0]).toMatchObject({
      path: "2.90/getting-started",
      href: "/docs/2.90/getting-started/",
    });
    expect(groups[0]?.items[0]?.children?.[0]).toMatchObject({
      path: "2.90/getting-started",
      href: "/docs/2.90/getting-started/#install",
    });
    expect(groups[0]?.items[1]?.href).toBe("/docs/2.90/getting-started/");
    expect(groups[0]?.items[2]?.href).toBe("/docs/2.90/getting-started/");
    expect(
      rewriteVersionedHeaderNavItems([{ text: "Guide", link: "/start" }], ctx)?.[0]?.link,
    ).toBe("/docs/2.90/getting-started/");
  });

  it("uses a version-local root for missing siblings", () => {
    const groups = rewriteVersionedNavGroups(
      [
        {
          title: "Guide",
          items: [{ title: "Missing", path: "missing", href: "/docs/missing/#part" }],
        },
      ],
      context(),
    );
    expect(groups[0]?.items[0]).toMatchObject({
      path: "2.90",
      href: "/docs/2.90/#part",
    });
  });

  it("never lets an alias shadow a canonical snapshot page", () => {
    const ctx = createVersionNavigationContext({
      prefix: "2.90",
      base: "/",
      pages: [
        {
          path: "guide",
          versionedPath: "2.90/guide",
          href: "/2.90/guide/",
          aliases: ["details"],
        },
        { path: "details", versionedPath: "2.90/details", href: "/2.90/details/" },
      ],
    });
    const groups = rewriteVersionedNavGroups(
      [{ title: "Guide", items: [{ title: "Details", path: "details", href: "/details/" }] }],
      ctx,
    );
    expect(groups[0]?.items[0]?.href).toBe("/2.90/details/");
  });

  it("leaves external, mail, hash-only, unsafe, and protocol-relative links untouched", () => {
    const hrefs = [
      "https://example.com/docs",
      "mailto:docs@example.com",
      "#install",
      "javascript:alert(1)",
      "data:text/plain,x",
      "//example.com/docs",
    ];
    const groups = rewriteVersionedNavGroups(
      [{ title: "Links", items: hrefs.map((href) => ({ title: href, path: "", href })) }],
      context(),
    );
    expect(groups[0]?.items.map((item) => item.href)).toEqual(hrefs);
  });

  it("composes locale and version prefixes without double-prefixing", () => {
    const ctx = context();
    const groups = rewriteVersionedNavGroups(
      [
        {
          title: "Guide",
          items: [
            {
              title: "Start",
              path: "ja/getting-started",
              href: "/docs/2.90/ja/getting-started/",
            },
          ],
        },
      ],
      ctx,
    );
    expect(groups[0]?.items[0]).toMatchObject({
      path: "2.90/ja/getting-started",
      href: "/docs/2.90/ja/getting-started/",
    });
    expect(unversionedPath("2.90/ja/getting-started", ctx)).toBe("ja/getting-started");
  });

  it("resolves locale siblings through pre-permalink source paths", () => {
    const groups = localizeNavGroups(
      [
        {
          title: "Guide",
          items: [{ title: "Start", path: "guide", href: "/docs/guide/", children: [] }],
        },
      ],
      {
        locale: "ja",
        locales: [{ code: "en" }, { code: "ja" }],
        defaultLocale: "en",
        hideDefaultLocale: true,
        base: "/docs/",
        pages: [
          { path: "getting-started", href: "/docs/2.90/getting-started/", aliases: ["guide"] },
          {
            path: "ja/getting-started",
            href: "/docs/2.90/ja/getting-started/",
            aliases: ["ja/guide"],
          },
        ],
      },
    );
    expect(groups[0]?.items[0]).toMatchObject({
      path: "ja/getting-started",
      href: "/docs/2.90/ja/getting-started/",
    });
  });

  it("supports a root base path", () => {
    const groups = rewriteVersionedNavGroups(
      [{ title: "Guide", items: [{ title: "Start", path: "getting-started", href: "/x" }] }],
      context("/"),
    );
    expect(groups[0]?.items[0]?.href).toBe("/2.90/getting-started/");
  });

  it("uses only version-local locale roots for missing translations", () => {
    expect(
      versionedLocaleRoots(context(), [{ code: "en" }, { code: "ja" }, { code: "fr" }], "en", true),
    ).toEqual({
      en: "/docs/2.90/",
      ja: "/docs/2.90/",
      fr: "/docs/2.90/",
    });
  });
});
