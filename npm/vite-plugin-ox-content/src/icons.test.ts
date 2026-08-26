import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import {
  collectIconFieldNames,
  collectIconNamesFromText,
  collectThemeIconNames,
  ICON_ASSET_DIR,
  ICON_CSS_NAME,
  iconStylesheetHref,
  parseIconName,
  resolveIconsOptions,
  withSelfHostedIconHead,
  writeSelfHostedIcons,
} from "./icons";
import { buildSsg } from "./ssg";
import { resolveTheme, themeToNapi } from "./theme";
import type { ResolvedIconsOptions } from "./types";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

const FIXTURE_COLLECTION = {
  prefix: "ox",
  width: 24,
  height: 24,
  icons: {
    mark: { body: '<path fill="currentColor" d="M2 2h20v20H2z"/>' },
    unused: { body: '<path fill="currentColor" d="M4 4h4v4H4z"/>' },
  },
};

async function tempDir(prefix: string): Promise<string> {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(dir);
  return dir;
}

async function writeFixtureCollection(root: string): Promise<void> {
  const dir = path.join(root, "node_modules", "@iconify-json", "ox");
  await fs.mkdir(dir, { recursive: true });
  await fs.writeFile(path.join(root, "package.json"), "{}\n");
  await fs.writeFile(path.join(dir, "icons.json"), JSON.stringify(FIXTURE_COLLECTION));
}

function enabledIcons(overrides: Partial<ResolvedIconsOptions> = {}): ResolvedIconsOptions {
  return {
    enabled: true,
    mode: "css-mask",
    syntax: "unocss",
    include: [],
    safelist: [],
    ...overrides,
  };
}

describe("resolveIconsOptions", () => {
  it("stays off when omitted", () => {
    expect(resolveIconsOptions(undefined).enabled).toBe(false);
    expect(resolveIconsOptions(false).enabled).toBe(false);
  });

  it("enables css-mask defaults", () => {
    expect(resolveIconsOptions(true)).toEqual(enabledIcons());
    expect(resolveIconsOptions({})).toEqual(enabledIcons());
  });

  it("keeps include names and safelist", () => {
    expect(
      resolveIconsOptions({
        include: ["ri:markdown-line"],
        safelist: ["carbon:checkbox"],
      }),
    ).toEqual(
      enabledIcons({
        include: ["ri:markdown-line"],
        safelist: ["carbon:checkbox"],
      }),
    );
  });
});

describe("icon name parsing", () => {
  it("accepts colon and unocss class forms", () => {
    expect(parseIconName("ri:markdown-line")).toEqual({ prefix: "ri", name: "markdown-line" });
    expect(parseIconName("icon-[line-md--rss]")).toEqual({ prefix: "line-md", name: "rss" });
    expect(parseIconName("https://example.com/x.svg")).toBeUndefined();
  });

  it("collects used names from source text", () => {
    const names = collectIconNamesFromText(
      `span.icon-[ox--mark] plus ox:mark and https://api.iconify.design/ox/unused.svg`,
    );
    expect([...names]).toEqual(["ox:mark"]);
  });

  it("collects frontmatter icon fields only", () => {
    const names = collectIconFieldNames('title: Home\nicon: "ox:mark"\nmentioned ox:unused\n');
    expect([...names]).toEqual(["ox:mark"]);
  });

  it("collects theme social Iconify names", () => {
    expect(
      collectThemeIconNames([
        { icon: "ox:mark", link: "https://example.com" },
        { icon: { svg: "<svg></svg>" }, link: "https://example.com" },
      ]),
    ).toEqual(["ox:mark"]);
  });
});

describe("theme embed", () => {
  it("prepends the icon stylesheet next to font tags", () => {
    const napi = themeToNapi(resolveTheme(undefined), undefined, "/docs/", true);
    expect(napi.embed?.head).toContain(iconStylesheetHref("/docs/"));
    expect(withSelfHostedIconHead({ head: "x" }, false)).toEqual({ head: "x" });
  });
});

describe("writeSelfHostedIcons", () => {
  it("emits CSS only for safelisted names from a local fixture", async () => {
    const root = await tempDir("ox-icons-safe-");
    const outDir = path.join(root, "dist");
    await writeFixtureCollection(root);

    const result = await writeSelfHostedIcons({
      options: enabledIcons({ safelist: ["ox:mark"] }),
      outDir,
      root,
    });

    expect(result.errors).toEqual([]);
    expect(result.names).toEqual(["ox:mark"]);
    const css = await fs.readFile(path.join(outDir, ICON_ASSET_DIR, ICON_CSS_NAME), "utf8");
    expect(css).toContain("icon-\\[ox--mark\\]");
    expect(css).toContain("currentColor");
    expect(css).not.toContain("ox--unused");
    expect(css).not.toContain("api.iconify.design");
  });

  it("resolves explicit include names without scanning", async () => {
    const root = await tempDir("ox-icons-include-");
    const outDir = path.join(root, "dist");
    await writeFixtureCollection(root);
    await fs.mkdir(path.join(root, "src"), { recursive: true });
    await fs.writeFile(path.join(root, "src", "page.md"), "icon-[ox--unused]\n");

    const result = await writeSelfHostedIcons({
      options: enabledIcons({ include: ["ox:mark"] }),
      outDir,
      root,
    });

    expect(result.names).toEqual(["ox:mark"]);
    const css = await fs.readFile(result.files[0]!, "utf8");
    expect(css).not.toContain("ox--unused");
  });

  it("diagnoses a missing collection and icon name", async () => {
    const root = await tempDir("ox-icons-miss-");
    await writeFixtureCollection(root);
    const missingCollection = await writeSelfHostedIcons({
      options: enabledIcons({ safelist: ["mdi:home"] }),
      outDir: path.join(root, "dist-a"),
      root,
    });
    expect(missingCollection.errors.join("\n")).toContain('missing Iconify collection "mdi"');

    const missingName = await writeSelfHostedIcons({
      options: enabledIcons({ safelist: ["ox:nope"] }),
      outDir: path.join(root, "dist-b"),
      root,
    });
    expect(missingName.errors.join("\n")).toContain('missing icon "ox:nope"');
  });
});

describe("SSG self-hosted icons", () => {
  it("keeps api.iconify.design off the entry page when icons are enabled", async () => {
    const root = await tempDir("ox-icons-ssg-");
    await writeFixtureCollection(root);
    await fs.mkdir(path.join(root, "content"), { recursive: true });
    await fs.writeFile(
      path.join(root, "content", "index.md"),
      [
        "---",
        "layout: entry",
        "title: Home",
        "features:",
        '  - icon: "ox:mark"',
        "    title: Mark",
        "---",
        "",
        "# Home",
        "",
      ].join("\n"),
    );

    const built = await buildSsg(
      createDocsResolvedOptions({
        icons: enabledIcons({ safelist: ["ox:mark"] }),
        search: { enabled: false, limit: 10, prefix: true, placeholder: "Search", hotkey: "/" },
        ssg: {
          ...createDocsResolvedOptions().ssg,
          theme: resolveTheme({
            socialLinks: [{ icon: "ox:mark", link: "https://example.com", ariaLabel: "Mark" }],
          }),
        },
      }),
      root,
    );

    expect(built.errors).toEqual([]);
    const html = await fs.readFile(path.join(root, "dist", "index.html"), "utf8");
    const css = await fs.readFile(path.join(root, "dist", ICON_ASSET_DIR, ICON_CSS_NAME), "utf8");
    expect(html).toContain("__ox_icons__/icons.css");
    expect(html).toContain("icon-[ox--mark]");
    expect(html).not.toContain("api.iconify.design");
    expect(css).toContain("icon-\\[ox--mark\\]");
    expect(css).not.toContain("ox--unused");
  });
});
