import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { resolveI18nOptions } from "./i18n";
import { localeHomeHref } from "./locale-nav";
import { buildSsg, resolveHeaderLogoHref } from "./ssg";
import { createVersionNavigationContext } from "./version-navigation";
import { createOxContentRouter } from "./dev-server";
import { resolveTheme } from "./theme";

const tempDirs: string[] = [];
afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

const locales = [{ code: "en" }, { code: "ja" }, { code: "zh-Hans-CN" }];
function jaOptions() {
  return {
    locale: "ja",
    locales,
    defaultLocale: "en",
    hideDefaultLocale: true,
    base: "/docs/",
    pages: [
      { path: "", href: "/docs/index.html" },
      { path: "ja", href: "/docs/ja/index.html" },
    ],
  };
}

describe("localeHomeHref", () => {
  it("uses the locale home and keeps the hidden default locale unprefixed", () => {
    expect(localeHomeHref(jaOptions())).toBe("/docs/ja/index.html");
    expect(localeHomeHref({ ...jaOptions(), locale: "en" })).toBe("/docs/index.html");
    expect(
      localeHomeHref({
        ...jaOptions(),
        locale: "zh-cn",
        locales: [...locales, { code: "zh-cn" }],
        pages: [...jaOptions().pages, { path: "zh-cn", href: "/docs/zh-cn/index.html" }],
      }),
    ).toBe("/docs/zh-cn/index.html");
  });

  it("prefixes the default locale when hideDefaultLocale is false", () => {
    expect(
      localeHomeHref({
        locale: "en",
        locales: [{ code: "en" }, { code: "ja" }],
        defaultLocale: "en",
        hideDefaultLocale: false,
        pages: [
          { path: "en", href: "/en/index.html" },
          { path: "ja", href: "/ja/index.html" },
        ],
        base: "/",
      }),
    ).toBe("/en/index.html");
  });

  it("keeps the logo inside the active documentation version", () => {
    const versionNavigation = createVersionNavigationContext({
      prefix: "2.90",
      base: "/docs/",
      pages: [
        { path: "", versionedPath: "2.90", href: "/docs/2.90/", sourcePath: "" },
        { path: "ja", versionedPath: "2.90/ja", href: "/docs/2.90/ja/", sourcePath: "ja" },
      ],
    });
    const shared = {
      locales: [{ code: "en" }, { code: "ja" }],
      defaultLocale: "en",
      hideDefaultLocale: true,
      pages: versionNavigation.pages,
      base: "/docs/",
    };
    expect(resolveHeaderLogoHref("/docs/", { ...shared, locale: "ja" }, versionNavigation)).toBe(
      "/docs/2.90/ja/",
    );
    expect(resolveHeaderLogoHref("/docs/", { ...shared, locale: "en" }, versionNavigation)).toBe(
      "/docs/2.90/",
    );
  });
  it("keeps versioned locale homes inside their route mount", () => {
    const versionNavigation = createVersionNavigationContext({
      prefix: "2.90",
      base: "/docs/",
      pages: [
        { path: "blog", versionedPath: "2.90/blog", href: "/docs/2.90/blog/", sourcePath: "" },
        {
          path: "blog/ja",
          versionedPath: "2.90/blog/ja",
          href: "/docs/2.90/blog/ja/",
          sourcePath: "ja",
        },
      ],
    });
    const shared = { ...jaOptions(), pages: versionNavigation.pages };
    expect(resolveHeaderLogoHref("/docs/", shared, versionNavigation, "blog")).toBe(
      "/docs/2.90/blog/ja/",
    );
    expect(
      resolveHeaderLogoHref("/docs/", { ...shared, locale: "en" }, versionNavigation, "blog"),
    ).toBe("/docs/2.90/blog/");
    expect(resolveHeaderLogoHref("/docs/", undefined, undefined, "blog")).toBe(
      "/docs/blog/index.html",
    );
  });
});

describe("header logo home", () => {
  it.each([undefined, "/blog/"])(
    "localizes SSG and development logos under mount %s",
    async (routePrefix) => {
      const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-locale-logo-"));
      tempDirs.push(root);
      const srcDir = path.join(root, "content");
      await fs.mkdir(path.join(srcDir, "ja"), { recursive: true });
      await fs.writeFile(path.join(srcDir, "index.md"), "# Home\n", "utf8");
      await fs.writeFile(path.join(srcDir, "guide.md"), "# Guide\n", "utf8");
      await fs.writeFile(path.join(srcDir, "ja", "index.md"), "# ホーム\n", "utf8");
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

      const options = createDocsResolvedOptions({
        base: "/docs/",
        i18n,
        ssg: {
          ...base.ssg,
          routePrefix,
          theme: resolveTheme({
            nav: [{ text: "Home", link: "/index.md" }],
          }),
        },
      });
      const result = await buildSsg(options, root);
      expect(result.errors).toEqual([]);

      const mount = routePrefix ? "blog/" : "";
      const jaGuide = await fs.readFile(
        path.join(root, "dist", mount, "ja", "guide", "index.html"),
        "utf8",
      );
      const enGuide = await fs.readFile(
        path.join(root, "dist", mount, "guide", "index.html"),
        "utf8",
      );
      expect(jaGuide).toContain(`href="/docs/${mount}ja/index.html" class="header-title"`);
      expect(enGuide).toContain(`href="/docs/${mount}index.html" class="header-title"`);
      expect(enGuide).not.toContain('href="/docs/en/index.html"');
      const router = createOxContentRouter(options, root);
      for (const [locale, expected] of [
        ["ja/", "ja/"],
        ["", ""],
      ]) {
        const response = await router.fetch(
          new Request(`http://localhost/docs/${mount}${locale}guide/`),
        );
        expect(response?.status).toBe(200);
        expect(await response!.text()).toContain(
          `href="/docs/${mount}${expected}index.html" class="header-title"`,
        );
      }
    },
  );
});
