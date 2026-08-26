import { mkdtemp, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import { render } from "svelte/server";
import { transformMarkdownWithSvelte } from "./transform";
import type { Component } from "svelte";
import type { ResolvedSvelteOptions } from "./types";

type GeneratedComponent = Component<Record<string, unknown>>;

describe("transformMarkdownWithSvelte", () => {
  it("turns registered components into islands and leaves fenced tags literal", async () => {
    const result = await transformMarkdownWithSvelte(
      [
        "---",
        "title: Svelte Guide",
        "draft: false",
        "---",
        "# Svelte Guide",
        "",
        '<Alert tone="info" active>Read **docs**.</Alert>',
        "",
        "```svelte",
        '<Alert tone="code" />',
        "```",
      ].join("\n"),
      "/repo/docs/svelte.md",
      createOptions(),
    );

    expect(result.frontmatter).toEqual({ title: "Svelte Guide", draft: false });
    expect(result.usedComponents).toEqual(["Alert"]);
    expect(result.code).toMatchSnapshot();
  });

  it("uses the static html path when no registered component is present", async () => {
    const result = await transformMarkdownWithSvelte(
      "# Plain\n\n<Unknown />",
      "/repo/docs/plain.md",
      createOptions(),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).toMatchSnapshot();
  });

  it("discovers nested, expression, and fragment islands from the MDX AST", async () => {
    const nested = await transformMarkdownWithSvelte(
      '<Callout>\n\n# Title\n\n<Badge title="hi" />\n\n</Callout>\n',
      "/repo/docs/nested.mdx",
      createOptions({
        components: {
          Alert: "./src/components/Alert.svelte",
          Callout: "./src/components/Callout.svelte",
          Badge: "./src/components/Badge.svelte",
        },
      }),
    );
    expect(nested.usedComponents).toEqual(["Callout", "Badge"]);
    expect(nested.code).toContain("Callout");
    expect(nested.code).toContain("Badge");
    expect(nested.code).toContain("initIslands");
    expect(nested.code).toMatchSnapshot();

    const expr = await transformMarkdownWithSvelte(
      "<Alert title={foo} count={count + 1} />\n",
      "/repo/docs/expr.mdx",
      createOptions(),
    );
    expect(expr.usedComponents).toEqual(["Alert"]);
    expect(expr.code).toContain("Alert");
    expect(expr.code).toMatchSnapshot();

    const fragment = await transformMarkdownWithSvelte(
      '<>\n<Alert tone="info" />\n</>\n',
      "/repo/docs/fragment.mdx",
      createOptions(),
    );
    expect(fragment.usedComponents).toEqual(["Alert"]);
    expect(fragment.code).toMatchSnapshot();
  });

  it("keeps fenced JSX literal and skips unregistered MDX components", async () => {
    const fenced = await transformMarkdownWithSvelte(
      [
        "# Guide",
        "",
        '<Alert tone="info" />',
        "",
        "```svelte",
        '<Alert tone="code" />',
        "```",
      ].join("\n"),
      "/repo/docs/fence.mdx",
      createOptions(),
    );
    expect(fenced.usedComponents).toEqual(["Alert"]);

    const mixed = await transformMarkdownWithSvelte(
      "# Plain\n\n<Alert />\n\n<Unknown />\n",
      "/repo/docs/mixed.mdx",
      createOptions(),
    );
    expect(mixed.usedComponents).toEqual(["Alert"]);
    expect(mixed.code).not.toContain("import Unknown");

    const unknownOnly = await transformMarkdownWithSvelte(
      "# Plain\n\n<Unknown />\n",
      "/repo/docs/unknown.mdx",
      createOptions(),
    );
    expect(unknownOnly.usedComponents).toEqual([]);
    expect(unknownOnly.code).not.toContain("initIslands");
  });

  it("honors disabled built-in embeds from framework options", async () => {
    const result = await transformMarkdownWithSvelte(
      '<GitHub repo="ubugeeei-prod/ox-content"></GitHub>',
      "/repo/docs/embed.md",
      createOptions({ embeds: { github: false, openGraph: false } }),
    );

    expect(result.code).toMatchSnapshot();
  });

  it("keeps MDX document props static unless explicitly enabled", async () => {
    const result = await transformMarkdownWithSvelte(
      "# Page\n\nHello {title}.\n",
      "/repo/docs/page.mdx",
      createOptions({ ssr: true }),
    );

    expect(result.code).toContain("Hello .");
    expect(result.code).not.toContain("__ox_mdx_document_prop");
    await withGeneratedModule(result.code, (Page) => {
      const rendered = render(Page, { props: { title: "Blog" } });
      expect(rendered.html).toContain("Hello .");
    });
  });

  it("renders opt-in MDX document text props during SSR", async () => {
    const result = await transformMarkdownWithSvelte(
      "# Page\n\nHello {title}.\n",
      "/repo/docs/page.mdx",
      createOptions({ mdxDocumentProps: true, ssr: true }),
    );

    expect(result.code).toContain("__ox_mdx_document_prop");
    expect(result.code).not.toContain("eval(");
    expect(result.code).not.toContain("new Function");
    await withGeneratedModule(result.code, (Page) => {
      const rendered = render(Page, { props: { title: "Blog" } });
      expect(stripSvelteComments(rendered.html)).toContain("Hello Blog.");
    });
  });

  it("passes host object and array props to document-local MDX components during SSR", async () => {
    const result = await transformMarkdownWithSvelte(
      "import PostList from './PostList.svelte'\n\n<PostList items={items} meta={meta} />\n",
      "/repo/docs/page.mdx",
      createOptions({ components: {}, mdxDocumentProps: true, ssr: true }),
    );
    const code = result.code.replace(
      "import PostList from './PostList.svelte';",
      [
        "const PostList = ($$renderer, $$props) => {",
        "  $$renderer.push('<pre>' + $$props.items.map((item) => item.title).join(',') + ':' + $$props.meta.kind + '</pre>');",
        "};",
      ].join("\n"),
    );

    expect(result.usedComponents).toEqual(["PostList"]);
    expect(result.code).toContain(
      'items: __ox_mdx_document_prop(__ox_mdx_props, ["items"], "items")',
    );
    await withGeneratedModule(code, (Page) => {
      const rendered = render(Page, {
        props: {
          items: [{ title: "Alpha" }, { title: "Beta" }],
          meta: { kind: "blog" },
        },
      });
      expect(rendered.html).toContain("<pre>Alpha,Beta:blog</pre>");
    });
  });

  it("throws a deterministic diagnostic for missing MDX document props", async () => {
    const result = await transformMarkdownWithSvelte(
      "# Page\n\nHello {title}.\n",
      "/repo/docs/page.mdx",
      createOptions({ mdxDocumentProps: true, ssr: true }),
    );

    await withGeneratedModule(result.code, (Page) => {
      expect(() => render(Page, { props: {} }).html).toThrow(
        '[ox-content-svelte] Missing MDX document prop "title" in /repo/docs/page.mdx for expression {title}.',
      );
    });
  });

  it("rejects unsupported MDX document prop expressions without eval fallback", async () => {
    await expect(
      transformMarkdownWithSvelte(
        "<Alert title={title + suffix} />\n",
        "/repo/docs/page.mdx",
        createOptions({ mdxDocumentProps: true, ssr: true }),
      ),
    ).rejects.toThrow(
      '[ox-content-svelte] Unsupported MDX document prop expression "{title + suffix}" for prop "title" in /repo/docs/page.mdx. Only identifiers and dotted property paths are supported.',
    );
  });
});

function createOptions(overrides: Partial<ResolvedSvelteOptions> = {}): ResolvedSvelteOptions {
  return {
    srcDir: "docs",
    outDir: "dist",
    base: "/",
    extensions: [".md", ".markdown", ".mdx"],
    gfm: true,
    frontmatter: true,
    toc: true,
    tocMaxDepth: 3,
    codeAnnotations: { enabled: false, metaKey: "annotate" },
    components: { Alert: "./src/components/Alert.svelte" },
    runes: true,
    embeds: { github: false, openGraph: false },
    root: "/repo",
    mdxDocumentProps: false,
    ...overrides,
  } as ResolvedSvelteOptions;
}

async function withGeneratedModule(
  code: string,
  callback: (component: GeneratedComponent) => void | Promise<void>,
): Promise<void> {
  const dir = await mkdtemp(path.join(os.tmpdir(), "ox-content-svelte-mdx-props-"));
  const file = path.join(dir, "page.mjs");
  await writeFile(file, code, "utf8");
  try {
    const mod = (await import(`${pathToFileURL(file).href}?t=${Date.now()}`)) as {
      default: GeneratedComponent;
    };
    await callback(mod.default);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}

function stripSvelteComments(html: string): string {
  return html.replace(/<!--[\s\S]*?-->/g, "");
}
