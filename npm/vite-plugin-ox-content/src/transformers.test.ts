import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { transformMarkdown } from "./transform";
import type { MarkdownNode, MarkdownTransformer, ResolvedOptions, TransformContext } from "./types";

function createResolvedOptions(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return createDocsResolvedOptions(overrides);
}

describe("transformers", () => {
  it("calls each transformer with the parsed tree and the page context", async () => {
    const seen: Array<{ name: string; type: string; context: TransformContext }> = [];
    const probe = (name: string): MarkdownTransformer => ({
      name,
      transform(ast, context) {
        seen.push({ name, type: ast.type, context });
        return ast;
      },
    });

    await transformMarkdown(
      "---\ntitle: Hi\n---\n\n# Heading\n\ntext\n",
      "docs/x.md",
      createResolvedOptions({ transformers: [probe("first"), probe("second")] }),
    );

    expect(seen.map((entry) => entry.name)).toEqual(["first", "second"]);
    expect(seen[0].type).toBe("root");
    expect(seen[0].context.filePath).toBe("docs/x.md");
    expect(seen[0].context.frontmatter).toEqual({ title: "Hi" });
  });

  it("renders the rewritten tree", async () => {
    const shout: MarkdownTransformer = {
      name: "shout",
      transform(ast) {
        const walk = (node: MarkdownNode): MarkdownNode =>
          node.type === "text"
            ? { ...node, value: String(node.value).toUpperCase() }
            : { ...node, children: (node.children ?? []).map(walk) };
        return walk(ast);
      },
    };

    const result = await transformMarkdown(
      "# Heading\n\nbody text\n",
      "docs/x.md",
      createResolvedOptions({ transformers: [shout] }),
    );

    expect(result.html).toContain("HEADING");
    expect(result.html).toContain("BODY TEXT");
  });

  it("passes each transformer the previous one's output", async () => {
    const append = (suffix: string): MarkdownTransformer => ({
      name: `append-${suffix}`,
      transform: (ast) => ({
        ...ast,
        children: [
          ...(ast.children ?? []),
          { type: "paragraph", children: [{ type: "text", value: suffix }] },
        ],
      }),
    });

    const result = await transformMarkdown(
      "start\n",
      "docs/x.md",
      createResolvedOptions({ transformers: [append("one"), append("two")] }),
    );

    expect(result.html).toContain("<p>start</p>");
    expect(result.html).toContain("<p>one</p>");
    expect(result.html).toContain("<p>two</p>");
    expect(result.html.indexOf("one")).toBeLessThan(result.html.indexOf("two"));
  });

  it("awaits an async transformer", async () => {
    const asyncTransformer: MarkdownTransformer = {
      name: "async",
      transform: async (ast) => {
        await Promise.resolve();
        return {
          ...ast,
          children: [{ type: "paragraph", children: [{ type: "text", value: "late" }] }],
        };
      },
    };

    const result = await transformMarkdown(
      "original\n",
      "docs/x.md",
      createResolvedOptions({ transformers: [asyncTransformer] }),
    );

    expect(result.html).toContain("late");
    expect(result.html).not.toContain("original");
  });

  it("keeps frontmatter and the table of contents across the hook", async () => {
    const passthrough: MarkdownTransformer = { name: "noop", transform: (ast) => ast };

    const result = await transformMarkdown(
      "---\ntitle: Kept\n---\n\n# One\n\n## Two\n",
      "docs/x.md",
      createResolvedOptions({ transformers: [passthrough] }),
    );

    expect(result.frontmatter).toEqual({ title: "Kept" });
    expect(result.toc.map((entry) => entry.text)).toEqual(["One"]);
    expect(result.toc[0].children?.map((entry) => entry.text)).toEqual(["Two"]);
  });

  it("produces the same output as no hook when the tree is untouched", async () => {
    const markdown =
      "---\ntitle: T\n---\n\n# One\n\n- [x] done\n\n| a | b |\n| - | - |\n| 1 | 2 |\n";
    const passthrough: MarkdownTransformer = { name: "noop", transform: (ast) => ast };

    const withHook = await transformMarkdown(
      markdown,
      "docs/x.md",
      createResolvedOptions({ transformers: [passthrough] }),
    );
    const withoutHook = await transformMarkdown(markdown, "docs/x.md", createResolvedOptions());

    expect(withHook.html).toBe(withoutHook.html);
    expect(withHook.toc).toEqual(withoutHook.toc);
  });

  it("skips a transformer that throws and keeps rendering the page", async () => {
    const warnings: unknown[][] = [];
    const original = console.warn;
    console.warn = (...args: unknown[]) => void warnings.push(args);

    try {
      const result = await transformMarkdown(
        "body\n",
        "docs/x.md",
        createResolvedOptions({
          transformers: [
            {
              name: "broken",
              transform() {
                throw new Error("boom");
              },
            },
          ],
        }),
      );

      expect(result.html).toContain("<p>body</p>");
      expect(JSON.stringify(warnings)).toContain("broken");
      expect(JSON.stringify(warnings)).toContain("boom");
    } finally {
      console.warn = original;
    }
  });

  it("skips a transformer that returns nothing", async () => {
    const warnings: unknown[][] = [];
    const original = console.warn;
    console.warn = (...args: unknown[]) => void warnings.push(args);

    try {
      const result = await transformMarkdown(
        "body\n",
        "docs/x.md",
        createResolvedOptions({
          transformers: [{ name: "forgetful", transform: (() => undefined) as never }],
        }),
      );

      expect(result.html).toContain("<p>body</p>");
      expect(JSON.stringify(warnings)).toContain("forgetful");
    } finally {
      console.warn = original;
    }
  });
});
