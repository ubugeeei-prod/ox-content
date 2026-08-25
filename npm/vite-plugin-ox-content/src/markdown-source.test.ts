import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { createDevServerCache, createDevServerMiddleware } from "./dev-server";
import { jsx } from "./jsx-html";
import {
  injectMarkdownSourceAlternate,
  markdownSourceHref,
  markdownSourceOutputPath,
  resolveMarkdownSourceOptions,
  shouldPublishMarkdownSource,
  writeMarkdownSourceFiles,
} from "./markdown-source";
import { usePageProps } from "./page-context";
import { resolvePublishStateOptions } from "./publish-state";
import { buildSsg, resolveSsgOptions } from "./ssg";
import type { ThemeComponent } from "./theme-renderer";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("resolveMarkdownSourceOptions", () => {
  it("disables the feature by default", () => {
    expect(resolveMarkdownSourceOptions(undefined)).toEqual({ enabled: false, alternate: true });
    expect(resolveMarkdownSourceOptions(false)).toEqual({ enabled: false, alternate: true });
    expect(resolveSsgOptions(undefined).markdownSource?.enabled).toBe(false);
  });

  it("enables defaults when true", () => {
    expect(resolveMarkdownSourceOptions(true)).toEqual({ enabled: true, alternate: true });
  });

  it("enables from an object and can turn the alternate link off", () => {
    expect(resolveMarkdownSourceOptions({})).toEqual({ enabled: true, alternate: true });
    expect(resolveMarkdownSourceOptions({ alternate: false })).toEqual({
      enabled: true,
      alternate: false,
    });
  });
});

describe("companion mapping", () => {
  it("maps published URLs onto .md companions honoring base", () => {
    expect(markdownSourceHref("/", "/")).toBe("/index.md");
    expect(markdownSourceHref("guide/intro", "/")).toBe("/guide/intro.md");
    expect(markdownSourceHref("ja/guide", "/docs/")).toBe("/docs/ja/guide.md");
    expect(markdownSourceOutputPath("/site/dist", "guide/intro")).toBe(
      path.join("/site/dist", "guide", "intro.md"),
    );
    expect(markdownSourceOutputPath("/site/dist", "../secret")).toBeUndefined();
  });

  it("never publishes draft or unlisted source", () => {
    const on = resolvePublishStateOptions(true);
    expect(shouldPublishMarkdownSource({ draft: true }, on)).toBe(false);
    expect(shouldPublishMarkdownSource({ unlisted: true }, on)).toBe(false);
    expect(shouldPublishMarkdownSource({ draft: true })).toBe(false);
    expect(shouldPublishMarkdownSource({ title: "Guide" }, on)).toBe(true);
  });

  it("injects an alternate link and leaves other HTML unchanged", () => {
    const html = "<html><head><title>G</title></head><body></body></html>";
    expect(injectMarkdownSourceAlternate(html, "/guide.md")).toContain(
      '<link rel="alternate" type="text/markdown" href="/guide.md">',
    );
    expect(injectMarkdownSourceAlternate("<p>bare</p>", "/guide.md")).toBe("<p>bare</p>");
  });
});

describe("SSG write", () => {
  it("writes companions and the alternate link when enabled", async () => {
    const source = "---\ntitle: Guide\n---\n# Guide\nBody.\n";
    const root = await makeSite({
      "index.md": "---\ntitle: Home\n---\n# Home\n",
      "guide.md": source,
    });
    const built = await buildSsg(ssgOptions({ markdownSource: true }), root);
    const companion = path.join(root, "dist", "guide.md");
    expect(built.files).toContain(companion);
    expect(await fs.readFile(companion, "utf8")).toBe(source);
    expect(await fs.readFile(path.join(root, "dist", "index.md"), "utf8")).toContain("title: Home");

    const html = await fs.readFile(path.join(root, "dist", "guide", "index.html"), "utf8");
    expect(html).toContain('<link rel="alternate" type="text/markdown" href="/guide.md">');
  });

  it("omits draft and unlisted companions even when HTML is written", async () => {
    const root = await makeSite({
      "guide.md": "---\ntitle: Guide\n---\n# Guide\n",
      "draft.md": "---\ntitle: Draft\ndraft: true\n---\n# Draft\nSecret.\n",
      "hidden.md": "---\ntitle: Hidden\nunlisted: true\n---\n# Hidden\nDirect only.\n",
    });
    const built = await buildSsg(
      ssgOptions({
        markdownSource: true,
        publishState: resolvePublishStateOptions({ now: "2026-08-24T00:00:00Z" }),
      }),
      root,
    );
    const names = built.files
      .filter((file) => file.endsWith(".md"))
      .map((file) => path.basename(file));
    expect(names).toEqual(["guide.md"]);
    expect(built.files.some((file) => file.includes(`${path.sep}hidden${path.sep}`))).toBe(true);
  });

  it("honors permalinks, base, and a custom HTML extension", async () => {
    const source = "---\ntitle: Intro\npermalink: /getting-started\n---\n# Intro\n";
    const root = await makeSite({ "guide/intro.md": source });
    const built = await buildSsg(
      ssgOptions({
        markdownSource: true,
        permalinks: { enabled: true },
        base: "/docs/",
        ssgExtension: ".htm",
      }),
      root,
    );
    const companion = path.join(root, "dist", "getting-started.md");
    expect(built.files).toContain(companion);
    expect(await fs.readFile(companion, "utf8")).toBe(source);
    const html = await fs.readFile(path.join(root, "dist", "getting-started", "index.htm"), "utf8");
    expect(html).toContain(
      '<link rel="alternate" type="text/markdown" href="/docs/getting-started.md">',
    );
  });

  it("exposes the source URL to a custom renderer", async () => {
    const root = await makeSite({ "guide.md": "---\ntitle: Guide\n---\n# Guide\n" });
    const Theme = () => {
      const page = usePageProps();
      return jsx("html", {
        children: [
          jsx("head", { children: jsx("title", { children: page.markdownSource ?? "" }) }),
          jsx("body", { children: "ok" }),
        ],
      });
    };
    await buildSsg(ssgOptions({ markdownSource: true, render: Theme }), root);
    const html = await fs.readFile(path.join(root, "dist", "guide", "index.html"), "utf8");
    expect(html).toContain("<title>/guide.md</title>");
  });

  it("leaves output unchanged when the feature is off", async () => {
    const files = { "guide.md": "---\ntitle: Guide\n---\n# Guide\n" };
    const offRoot = await makeSite(files);
    const onRoot = await makeSite(files);
    const off = await buildSsg(ssgOptions({ markdownSource: false }), offRoot);
    const on = await buildSsg(ssgOptions({ markdownSource: true }), onRoot);
    const offNames = relativeNames(off.files, offRoot);
    const onNames = relativeNames(on.files, onRoot);
    expect(offNames.some((name) => name.endsWith(".md"))).toBe(false);
    expect(onNames).toContain(path.join("dist", "guide.md"));
    const offHtml = await fs.readFile(path.join(offRoot, "dist", "guide", "index.html"), "utf8");
    expect(offHtml).not.toContain('rel="alternate"');
    expect(offHtml).not.toContain("text/markdown");
  });

  it("writes nothing when the resolver is off", async () => {
    const result = await writeMarkdownSourceFiles({
      outDir: "/tmp",
      base: "/",
      pages: [{ inputPath: "a.md", source: "# A\n", urlPath: "a", frontmatter: {} }],
    });
    expect(result).toEqual({ files: [], errors: [] });
  });
});

describe("dev server", () => {
  it("serves published source and hides drafts", async () => {
    const source = "---\ntitle: Guide\n---\n# Guide\nBody.\n";
    const root = await makeSite({
      "guide.md": source,
      "draft.md": "---\ntitle: Draft\ndraft: true\n---\n# Draft\n",
    });
    const cache = createDevServerCache();
    const middleware = createDevServerMiddleware(ssgOptions({ markdownSource: true }), root, cache);

    const published = await dispatch(middleware, "/guide.md");
    expect(published.status).toBe(200);
    expect(published.type).toBe("text/markdown; charset=utf-8");
    expect(published.body).toBe(source);

    const hidden = await dispatch(middleware, "/draft.md");
    expect(hidden.status).toBe(404);
    expect(hidden.body).toBe("");
  });
});

async function makeSite(files: Record<string, string>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-md-source-"));
  tempDirs.push(root);
  for (const [name, source] of Object.entries(files)) {
    const full = path.join(root, "content", name);
    await fs.mkdir(path.dirname(full), { recursive: true });
    await fs.writeFile(full, source);
  }
  return root;
}

function ssgOptions(
  input: {
    markdownSource?: boolean | { alternate?: boolean };
    publishState?: ReturnType<typeof resolvePublishStateOptions>;
    permalinks?: { enabled: boolean };
    base?: string;
    ssgExtension?: string;
    render?: ThemeComponent;
  } = {},
) {
  const base = createDocsResolvedOptions();
  return createDocsResolvedOptions({
    base: input.base ?? "/",
    permalinks: input.permalinks ?? { enabled: false },
    publishState: input.publishState,
    ssg: {
      ...base.ssg,
      bare: !input.render,
      extension: input.ssgExtension ?? ".html",
      markdownSource: resolveMarkdownSourceOptions(input.markdownSource),
      ...(input.render ? { render: input.render } : {}),
    },
  });
}

function relativeNames(files: string[], root: string): string[] {
  return files.map((file) => path.relative(root, file));
}

function dispatch(
  middleware: ReturnType<typeof createDevServerMiddleware>,
  url: string,
): Promise<{ status: number; type?: string; body: string }> {
  return new Promise((resolve, reject) => {
    const headers: Record<string, string> = {};
    const res = {
      statusCode: 200,
      setHeader(name: string, value: string) {
        headers[name.toLowerCase()] = value;
      },
      end(body?: string) {
        resolve({ status: this.statusCode, type: headers["content-type"], body: body ?? "" });
      },
    };
    middleware({ url } as never, res as never, () => reject(new Error(`next() called for ${url}`)));
  });
}
