import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { buildSearchIndex } from "./search";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions, TocEntry } from "./types";

describe("conditional block transform", () => {
  it("leaves conditional source literal unless opted in", async () => {
    const result = await transformMarkdown(
      '::: if runtime == "node"\nNode\n::: else\nBrowser\n:::\n',
      "docs/runtime.md",
      createResolvedOptions(),
    );
    expect(result.html).toContain("runtime");
    expect(result.html).toContain("Node");
    expect(result.html).toContain("Browser");
  });

  it("renders only the selected branch into html, toc, and module code", async () => {
    const result = await transformMarkdown(
      [
        "# Runtime",
        "",
        '::: if runtime == "node"',
        "## Node only",
        "Selected branch.",
        "::: else",
        "## Browser only",
        "Hidden branch.",
        ":::",
        "",
      ].join("\n"),
      "docs/runtime.md",
      createResolvedOptions({
        conditionalBlocks: { enabled: true, values: { runtime: "node" } },
      }),
    );
    expect(result.html).toContain("Node only");
    expect(result.html).not.toContain("Browser only");
    expect(result.code).toContain("Selected branch.");
    expect(result.code).not.toContain("Hidden branch.");
    expect(tocTexts(result.toc)).toEqual(["Runtime", "Node only"]);
  });

  it("lets page frontmatter override bare config identifiers", async () => {
    const result = await transformMarkdown(
      [
        "---",
        "runtime: browser",
        "---",
        '::: if runtime == "browser"',
        "Frontmatter branch.",
        '::: else if config.runtime == "node"',
        "Config branch.",
        ":::",
        "",
      ].join("\n"),
      "docs/runtime.md",
      createResolvedOptions({
        conditionalBlocks: { enabled: true, values: { runtime: "node" } },
      }),
    );
    expect(result.html).toContain("Frontmatter branch.");
    expect(result.html).not.toContain("Config branch.");
  });

  it("reports malformed conditions and can fall through to else", async () => {
    const { result, warnings } = await captureTransformWarnings(() =>
      transformMarkdown(
        '::: if runtime = "node"\nBroken.\n::: else\nFallback.\n:::\n',
        "docs/runtime.md",
        createResolvedOptions({
          conditionalBlocks: { enabled: true, values: { runtime: "node" } },
        }),
      ),
    );
    expect(flattenWarnings(warnings)).toContain("Conditional block expression");
    expect(result.html).toContain("Fallback.");
    expect(result.html).not.toContain("Broken.");
  });

  it("excludes non-selected branches from the search index", async () => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-conditional-"));
    try {
      const contentDir = path.join(root, "content");
      await fs.mkdir(contentDir);
      await fs.writeFile(
        path.join(contentDir, "runtime.md"),
        [
          "# Runtime",
          "",
          '::: if runtime in ["node", "deno"]',
          "## Node only",
          "search-visible phrase",
          "::: else",
          "## Browser only",
          "search-hidden phrase",
          ":::",
          "",
        ].join("\n"),
        "utf8",
      );
      const index = JSON.parse(
        await buildSearchIndex(contentDir, "/docs/", [".md"], undefined, [], undefined, {
          enabled: true,
          values: { runtime: "node" },
        }),
      ) as { documents: Array<{ body: string; headings: string[] }> };
      expect(index.documents).toHaveLength(1);
      expect(index.documents[0]?.body).toContain("search-visible");
      expect(index.documents[0]?.body).not.toContain("search-hidden");
      expect(index.documents[0]?.headings).toContain("Node only");
      expect(index.documents[0]?.headings).not.toContain("Browser only");
    } finally {
      await fs.rm(root, { recursive: true, force: true });
    }
  });
});

function createResolvedOptions(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return createDocsResolvedOptions({ highlight: false, ...overrides });
}

async function captureTransformWarnings<T>(
  run: () => Promise<T>,
): Promise<{ result: T; warnings: unknown[][] }> {
  const warnings: unknown[][] = [];
  const original = console.warn;
  console.warn = (...args: unknown[]) => {
    warnings.push(args);
  };
  try {
    return { result: await run(), warnings };
  } finally {
    console.warn = original;
  }
}

function flattenWarnings(warnings: unknown[][]): string {
  return warnings.flatMap((args) => args.map(String)).join("\n");
}

function tocTexts(entries: TocEntry[]): string[] {
  const texts: string[] = [];
  for (const entry of entries) {
    texts.push(entry.text);
    texts.push(...tocTexts(entry.children ?? []));
  }
  return texts;
}
