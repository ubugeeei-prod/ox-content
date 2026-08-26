import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { resolveI18nOptions } from "./i18n";
import { buildSsg } from "./ssg";
import { resolveTheme } from "./theme";
import { resolveVersionsOptions } from "./versions";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("versioned SSG navigation", () => {
  it("keeps generated navigation, breadcrumbs, pager, and search inside three snapshot pages", async () => {
    const root = await makeVersionedSite(
      pageSources(["alpha", "beta", "gamma"], "Live"),
      pageSources(["alpha", "beta", "gamma"], "Frozen"),
    );
    const base = createDocsResolvedOptions();
    const result = await buildSsg(
      createDocsResolvedOptions({
        base: "/docs/",
        versions: enabledVersions(),
        ssg: { ...base.ssg, pagination: true, breadcrumbs: true },
      }),
      root,
    );

    expect(result.errors).toEqual([]);
    for (const name of ["alpha", "beta", "gamma"]) {
      const html = await readSnapshot(root, name);
      for (const sibling of ["alpha", "beta", "gamma"]) {
        expect(html).toContain(`href="/docs/2.90/${sibling}/"`);
        expect(html).not.toContain(`href="/docs/${sibling}/index.html"`);
      }
      expect(html).toContain(`data-ox-search-index="/docs/2.90/search-index.json"`);
      expect(html).toContain(`class="nav-link active">${title(name)}</a>`);
    }
    const beta = await readSnapshot(root, "beta");
    expect(beta).toContain('aria-label="Breadcrumb"');
    expect(beta).toContain('class="pager-link pager-link--prev"');
    expect(beta).toContain('class="pager-link pager-link--next"');
    expect(beta).toContain('class="ox-breadcrumbs-link" href="/docs/2.90/"');
  });

  it("composes base, locale, permalink, alias, manual sidebar, and fallback routes", async () => {
    const sources = {
      "guide.md":
        "---\ntitle: Guide\npermalink: /getting-started\naliases: [/start]\n---\n# Guide\n",
      "details.md": "---\ntitle: Details\n---\n# Details\n",
      "only-en.md": "---\ntitle: English only\n---\n# English only\n",
      "ja/guide.md":
        "---\ntitle: ガイド\npermalink: /ja/getting-started\naliases: [/ja/start]\n---\n# ガイド\n",
      "ja/details.md": "---\ntitle: 詳細\n---\n# 詳細\n",
    };
    const root = await makeVersionedSite(sources, sources);
    const base = createDocsResolvedOptions();
    const i18n = resolveI18nOptions({
      enabled: true,
      defaultLocale: "en",
      hideDefaultLocale: true,
      check: false,
      locales: [
        { code: "en", name: "English" },
        { code: "ja", name: "日本語" },
      ],
    });
    expect(i18n).not.toBe(false);

    const result = await buildSsg(
      createDocsResolvedOptions({
        base: "/docs/",
        versions: enabledVersions(),
        i18n,
        permalinks: { enabled: true },
        redirects: {
          enabled: true,
          map: { "/old-guide": "/getting-started" },
          headers: false,
          json: false,
          allowExternal: false,
        },
        ssg: {
          ...base.ssg,
          pagination: true,
          breadcrumbs: true,
          localeSwitcher: true,
          theme: resolveTheme({
            nav: [{ text: { en: "Guide", ja: "ガイド" }, link: "/guide.md" }],
            sidebar: [
              {
                text: { en: "Guide", ja: "ガイド" },
                items: [
                  { text: { en: "Start", ja: "開始" }, link: "/guide.md" },
                  { text: { en: "Alias", ja: "別名" }, link: "/start" },
                  { text: { en: "Details", ja: "詳細" }, link: "/details.md" },
                  { text: { en: "Missing", ja: "欠落" }, link: "/missing.md" },
                  { text: "External", link: "https://example.com/docs" },
                  { text: "Mail", link: "mailto:docs@example.com" },
                  { text: "Section", link: "#section" },
                ],
              },
            ],
          }),
        },
      }),
      root,
    );

    expect(result.errors).toEqual([]);
    const japanese = await fs.readFile(
      path.join(root, "dist", "2.90", "ja", "getting-started", "index.html"),
      "utf8",
    );
    expect(japanese).toContain('href="/docs/2.90/ja/getting-started/"');
    expect(japanese).toContain('href="/docs/2.90/ja/details/"');
    expect(japanese).toContain('href="/docs/2.90/"');
    expect(japanese).toContain('href="https://example.com/docs"');
    expect(japanese).toContain('href="mailto:docs@example.com"');
    expect(japanese).toContain('href="#section"');
    expect(japanese).toContain('class="nav-link active">開始</a>');
    expect(japanese).toContain('data-ox-search-index="/docs/2.90/search-index.json"');

    const englishOnly = await fs.readFile(
      path.join(root, "dist", "2.90", "only-en", "index.html"),
      "utf8",
    );
    expect(englishOnly).toContain('<a href="/docs/2.90/" hreflang="ja"');

    const current = await fs.readFile(
      path.join(root, "dist", "getting-started", "index.html"),
      "utf8",
    );
    expect(current).toContain('href="/docs/guide/index.html"');
    expect(current).not.toContain('class="nav-link" href="/docs/2.90/guide/"');
  });
});

function enabledVersions() {
  return resolveVersionsOptions({
    current: "3.0.0-alpha",
    entries: [
      { id: "3.0.0-alpha", label: "3.0.0-alpha", prefix: "", banner: "unreleased" },
      { id: "2.90.0", label: "2.90.0", prefix: "2.90", dir: "versions/2.90" },
    ],
  });
}

async function makeVersionedSite(
  current: Record<string, string>,
  snapshot: Record<string, string>,
): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-version-nav-"));
  tempDirs.push(root);
  await writeSources(path.join(root, "content"), current);
  await writeSources(path.join(root, "versions", "2.90"), snapshot);
  return root;
}

async function writeSources(dir: string, sources: Record<string, string>): Promise<void> {
  for (const [relative, source] of Object.entries(sources)) {
    const file = path.join(dir, relative);
    await fs.mkdir(path.dirname(file), { recursive: true });
    await fs.writeFile(file, source, "utf8");
  }
}

function pageSources(names: string[], prefix: string): Record<string, string> {
  return Object.fromEntries(
    names.map((name) => [
      name + ".md",
      `---\ntitle: ${title(name)}\n---\n# ${title(name)}\n${prefix}.\n`,
    ]),
  );
}

function title(value: string): string {
  return value[0]!.toUpperCase() + value.slice(1);
}

function readSnapshot(root: string, name: string): Promise<string> {
  return fs.readFile(path.join(root, "dist", "2.90", name, "index.html"), "utf8");
}
