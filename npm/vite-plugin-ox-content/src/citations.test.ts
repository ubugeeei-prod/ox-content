import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { resolveOptions } from "./resolve-options";
import { renderMarkdown } from "./render-markdown";
import { buildSearchIndex } from "./search";
import { buildSsg } from "./ssg";
import type { ResolvedOptions } from "./types";
import { transformMarkdown } from "./transform";

const tempDirs: string[] = [];
const originalWarn = console.warn;

afterEach(async () => {
  console.warn = originalWarn;
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("citations", () => {
  it("renders repeated and grouped CSL citations with one bibliography entry per key", async () => {
    const { root, bibliography } = await bibliographyRoot([
      csl("rfc9110", "HTTP Semantics", "Roy T.", "Fielding", 2022, {
        URL: "https://www.rfc-editor.org/rfc/rfc9110",
      }),
      csl("smith2024", "Static Documentation Systems", "Ada", "Smith", 2024),
    ]);

    const result = await transformMarkdown(
      "HTTP is cited once [@rfc9110] and then grouped [@rfc9110; -@smith2024].",
      "/virtual/citations.md",
      options(root, bibliography),
    );

    expect(result.citations).toMatchObject([
      { key: "rfc9110", index: 1, label: "1", suppressAuthor: false },
      { key: "rfc9110", index: 1, label: "1", suppressAuthor: false },
      { key: "smith2024", index: 2, label: "2", suppressAuthor: true },
    ]);
    expect(result.bibliography).toHaveLength(2);
    expect(result.html).toContain('<span class="ox-cite" role="group"');
    expect(result.html).toContain('<section class="ox-bibliography"');
    expect(result.html.match(/class="ox-bibliography__item"/g)).toHaveLength(2);
    expect(result.html).toContain("Roy T. Fielding (2022).");
    expect(result.code).toContain("export const citations =");
    expect(result.code).toContain("export const bibliography =");
    expect(result.html).toMatchInlineSnapshot(`
      "<p>HTTP is cited once <span class="ox-cite" role="group" aria-label="Citations 1"><a class="ox-cite__ref" id="cite-rfc9110-m9d8xx-1" href="#ref-rfc9110-m9d8xx" data-ox-citation-key="rfc9110" data-ox-citation-index="1" aria-label="Citation 1: HTTP Semantics">[1]</a></span> and then grouped <span class="ox-cite" role="group" aria-label="Citations 1, 2">[<a class="ox-cite__ref" id="cite-rfc9110-m9d8xx-2" href="#ref-rfc9110-m9d8xx" data-ox-citation-key="rfc9110" data-ox-citation-index="1" aria-label="Citation 1: HTTP Semantics">1</a>; <a class="ox-cite__ref" id="cite-smith2024-u3g7cc-3" href="#ref-smith2024-u3g7cc" data-ox-citation-key="smith2024" data-ox-citation-index="2" aria-label="Citation 2: Static Documentation Systems">2</a>]</span>.</p>
      <section class="ox-bibliography" aria-labelledby="ox-bibliography-title"><h2 class="ox-bibliography__title" id="ox-bibliography-title">References</h2><ol class="ox-bibliography__list"><li class="ox-bibliography__item" id="ref-rfc9110-m9d8xx" data-ox-citation-key="rfc9110" value="1">Roy T. Fielding (2022). <cite>HTTP Semantics</cite>. <a class="ox-bibliography__url" href="https://www.rfc-editor.org/rfc/rfc9110">https://www.rfc-editor.org/rfc/rfc9110</a>.</li><li class="ox-bibliography__item" id="ref-smith2024-u3g7cc" data-ox-citation-key="smith2024" value="2">Ada Smith (2024). <cite>Static Documentation Systems</cite>.</li></ol></section>"
    `);
  });

  it("leaves citation-looking prose and footnotes unchanged when disabled", async () => {
    const result = await renderMarkdown(
      ["Literal [@rfc9110].", "", "A footnote.[^note]", "", "[^note]: Footnote body."].join("\n"),
      "/virtual/disabled.md",
      { highlight: false, citations: false, footnotes: true },
    );

    expect(result.html).toContain("[@rfc9110]");
    expect(result.html).toContain("Footnote body");
    expect(result.html).not.toContain("ox-cite");
    expect(result.bibliography).toEqual([]);
  });

  it("does not rewrite citation syntax inside code, protected HTML, or comments", async () => {
    const { root, bibliography } = await bibliographyRoot([csl("rfc9110", "HTTP Semantics")]);
    const result = await transformMarkdown(
      [
        "Inline `[@rfc9110]` stays code.",
        "",
        "```md",
        "[@rfc9110]",
        "```",
        "",
        "<pre>[@rfc9110]</pre>",
        "<!-- [@rfc9110] -->",
        "",
        "Normal [@rfc9110].",
      ].join("\n"),
      "/virtual/protected.md",
      options(root, bibliography),
    );

    expect(result.html).toContain("<code>[@rfc9110]</code>");
    expect(result.html).toContain('<pre><code class="language-md">[@rfc9110]');
    expect(result.html).toContain("<pre>[@rfc9110]</pre>");
    expect(result.html).toContain("<!-- [@rfc9110] -->");
    expect(result.html.match(/class="ox-cite__ref"/g)).toHaveLength(1);
  });

  it("reports missing, malformed, duplicate, and escaping bibliography problems", async () => {
    const { root, bibliography } = await bibliographyRoot([csl("rfc9110", "HTTP Semantics")]);
    await expect(
      transformMarkdown("Missing [@missing].", "/virtual/missing.md", options(root, bibliography)),
    ).rejects.toThrow('missing citation key "missing"');

    await expect(
      transformMarkdown(
        "Malformed [@bad key].",
        "/virtual/malformed.md",
        options(root, bibliography),
      ),
    ).rejects.toThrow('malformed citation "[@bad key]"');

    const duplicate = await bibliographyRoot([
      csl("rfc9110", "HTTP Semantics"),
      csl("rfc9110", "Duplicate HTTP Semantics"),
    ]);
    await expect(
      transformMarkdown(
        "Duplicate [@rfc9110].",
        "/virtual/duplicate.md",
        options(duplicate.root, duplicate.bibliography),
      ),
    ).rejects.toThrow('duplicate citation key "rfc9110"');

    await expect(
      transformMarkdown(
        "Remote [@rfc9110].",
        "/virtual/remote.md",
        options(root, "https://example.com/refs.json"),
      ),
    ).rejects.toThrow("must be a local file path");
  });

  it("can warn and leave unresolved citation references literal", async () => {
    const warnings: string[] = [];
    console.warn = (message?: unknown) => warnings.push(String(message));
    const result = await transformMarkdown(
      "Missing [@missing].",
      "/virtual/warn.md",
      createDocsResolvedOptions({
        highlight: false,
        citations: {
          enabled: true,
          bibliography: [],
          missing: "warn",
        },
      }),
    );

    expect(result.html).toContain("[@missing]");
    expect(result.citations).toEqual([]);
    expect(warnings[0]).toContain('missing citation key "missing"');
  });

  it("keeps grouped warn-mode misses atomic", async () => {
    const warnings: string[] = [];
    console.warn = (message?: unknown) => warnings.push(String(message));
    const { root, bibliography } = await bibliographyRoot([csl("rfc9110", "HTTP Semantics")]);
    const resolved = options(root, bibliography);
    const result = await transformMarkdown("Mixed [@rfc9110; @missing].", "/virtual/warn.md", {
      ...resolved,
      citations: { ...resolved.citations, missing: "warn" },
    });

    expect(result.html).toContain("[@rfc9110; @missing]");
    expect(result.citations).toEqual([]);
    expect(result.bibliography).toEqual([]);
    expect(warnings[0]).toContain('missing citation key "missing"');
  });

  it("does not treat citation keys as cross-reference syntax", async () => {
    const { root, bibliography } = await bibliographyRoot([
      csl("rfc9110", "HTTP Semantics"),
      csl("fig-study", "Figure Studies"),
    ]);
    const result = await transformMarkdown(
      "Cite figure research [@rfc9110; @fig-study].",
      "/virtual/both.md",
      {
        ...options(root, bibliography),
        crossReferences: {
          enabled: true,
          missing: "error",
          duplicates: "error",
          mismatches: "error",
          labels: { figure: "Figure", table: "Table", section: "Section" },
        },
      },
    );

    expect(result.html).toContain('data-ox-citation-key="fig-study"');
    expect(result.citations.map((citation) => citation.key)).toEqual(["rfc9110", "fig-study"]);
    expect(result.crossReferences).toEqual([]);
  });

  it("resolves disabled, boolean, and object option forms", () => {
    expect(resolveOptions({ citations: false }).citations.enabled).toBe(false);
    expect(resolveOptions({ citations: true }).citations).toMatchObject({
      enabled: true,
      bibliography: [],
      appendBibliography: true,
      missing: "error",
    });
    expect(
      resolveOptions({
        citations: {
          bibliography: ["refs.json"],
          appendBibliography: false,
          malformed: "warn",
          bibliographyTitle: "Works Cited",
        },
      }).citations,
    ).toMatchObject({
      enabled: true,
      bibliography: ["refs.json"],
      appendBibliography: false,
      malformed: "warn",
      bibliographyTitle: "Works Cited",
    });
  });

  it("includes generated bibliography in SSG HTML only when enabled", async () => {
    const enabledRoot = await tempRoot("ox-content-citations-ssg-on-");
    await writeSsgFixture(enabledRoot);
    const enabledOptions = createDocsResolvedOptions({
      srcDir: "content",
      outDir: "dist",
      highlight: false,
      search: { enabled: false, limit: 10, prefix: true, placeholder: "Search", hotkey: "/" },
      citations: {
        enabled: true,
        bibliography: ["refs.json"],
        rootDir: enabledRoot,
        appendBibliography: true,
        missing: "error",
        duplicates: "error",
        malformed: "error",
        bibliographyTitle: "References",
      },
    });
    await buildSsg(enabledOptions, enabledRoot);
    const enabledHtml = await fs.readFile(path.join(enabledRoot, "dist", "index.html"), "utf8");
    const enabledIndex = await searchIndexBody(enabledRoot, enabledOptions);
    expect(enabledHtml).toContain("HTTP Semantics");
    expect(enabledIndex).toContain("HTTP Semantics");
    expect(enabledHtml).toContain("ox-bibliography");

    const disabledRoot = await tempRoot("ox-content-citations-ssg-off-");
    await writeSsgFixture(disabledRoot);
    const disabledOptions = createDocsResolvedOptions({
      srcDir: "content",
      outDir: "dist",
      highlight: false,
      search: { enabled: false, limit: 10, prefix: true, placeholder: "Search", hotkey: "/" },
    });
    await buildSsg(disabledOptions, disabledRoot);
    const disabledHtml = await fs.readFile(path.join(disabledRoot, "dist", "index.html"), "utf8");
    const disabledIndex = await searchIndexBody(disabledRoot, disabledOptions);
    expect(disabledHtml).toContain("[@rfc9110]");
    expect(disabledIndex).not.toContain("HTTP Semantics");
    expect(disabledHtml).not.toContain("ox-bibliography");
  });
});

function options(
  rootDir: string,
  bibliography: string | string[],
  overrides: Partial<ResolvedOptions> = {},
): ResolvedOptions {
  return createDocsResolvedOptions({
    highlight: false,
    citations: {
      enabled: true,
      bibliography: toBibliographyList(bibliography),
      rootDir,
      appendBibliography: true,
      missing: "error",
      duplicates: "error",
      malformed: "error",
      bibliographyTitle: "References",
    },
    ...overrides,
  });
}

function toBibliographyList(value: string | string[]): string[] {
  return Array.isArray(value) ? value : [value];
}

async function bibliographyRoot(
  entries: unknown[],
): Promise<{ root: string; bibliography: string }> {
  const root = await tempRoot("ox-content-citations-");
  const bibliography = "refs.json";
  await fs.writeFile(path.join(root, bibliography), JSON.stringify(entries), "utf8");
  return { root, bibliography };
}

async function writeSsgFixture(root: string): Promise<void> {
  await fs.mkdir(path.join(root, "content"), { recursive: true });
  await fs.writeFile(
    path.join(root, "content", "index.md"),
    "# Guide\n\nCite the spec [@rfc9110].\n",
    "utf8",
  );
  await fs.writeFile(
    path.join(root, "refs.json"),
    JSON.stringify([csl("rfc9110", "HTTP Semantics", "Roy T.", "Fielding", 2022)]),
    "utf8",
  );
}

async function searchIndexBody(root: string, options: ResolvedOptions): Promise<string> {
  const json = await buildSearchIndex(
    path.join(root, options.srcDir),
    "/",
    [".md"],
    undefined,
    [],
    undefined,
    undefined,
    options.citations,
  );
  const index = JSON.parse(json) as { documents: Array<{ body: string }> };
  return index.documents.map((document) => document.body).join("\n");
}

async function tempRoot(prefix: string): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(root);
  return root;
}

function csl(
  id: string,
  title: string,
  given?: string,
  family?: string,
  year?: number,
  extra: Record<string, unknown> = {},
) {
  return {
    id,
    title,
    author: family ? [{ given, family }] : undefined,
    issued: year ? { "date-parts": [[year]] } : undefined,
    ...extra,
  };
}
