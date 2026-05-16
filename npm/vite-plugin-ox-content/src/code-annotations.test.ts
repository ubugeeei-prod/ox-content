import { describe, expect, it } from "vite-plus/test";
import type { ResolvedOptions } from "./types";
import { transformMarkdown } from "./transform";

describe("code annotations", () => {
  it("preserves pre and line classes after syntax highlighting", async () => {
    const markdown = `\`\`\`ts annotate="highlight:1;warning:2;error:3"
const first = 1;
const second = 2;
throw new Error("boom");
\`\`\`
`;

    const result = await transformMarkdown(
      markdown,
      "docs/code-annotations.md",
      createResolvedOptions(),
    );

    expect(result.html).toContain("ox-code-block--annotated");
    expect(result.html).toContain("has-highlighted");
    expect(result.html).toContain(
      'class="line ox-code-line ox-code-line--highlight highlighted" data-line="1"',
    );
    expect(result.html).toContain(
      'class="line ox-code-line ox-code-line--warning highlighted warning" data-line="2"',
    );
    expect(result.html).toContain(
      'class="line ox-code-line ox-code-line--error highlighted error" data-line="3"',
    );
    expect(result.html).toContain('class="language-ts" data-language="ts"');
  });

  it("keeps non-annotated highlighted blocks unchanged", async () => {
    const markdown = `\`\`\`ts
const value = 1;
const next = 2;
\`\`\`
`;

    const result = await transformMarkdown(markdown, "docs/plain-code.md", createResolvedOptions());

    expect(result.html).not.toContain("ox-code-line--warning");
    expect(result.html).not.toContain("ox-code-block--annotated");
    expect(result.html).toContain('class="language-ts" data-language="ts"');
  });

  it("supports VitePress-style fence metadata", async () => {
    const markdown = `\`\`\`ts:line-numbers=7 {1,3} [config.ts]
const first = true;
const second = false;
const third = true;
\`\`\`
`;

    const result = await transformMarkdown(
      markdown,
      "docs/vitepress-meta.md",
      createResolvedOptions({
        codeAnnotations: {
          enabled: true,
          notation: "vitepress",
          metaKey: "annotate",
          defaultLineNumbers: false,
        },
      }),
    );

    expect(result.html).toContain('data-code-title="config.ts"');
    expect(result.html).toContain('data-line-number-start="7"');
    expect(result.html).toContain('data-line-number="7"');
    expect(result.html).toContain('data-line-number="9"');
    expect(result.html).toContain("ox-code-line--highlight");
    expect(result.html).toContain('class="language-ts" data-language="ts"');
  });

  it("supports VitePress-style inline directives", async () => {
    const markdown = `\`\`\`ts
// [!code focus:2]
const first = true;
const second = false;
console.log("before") // [!code --]
console.log("after") // [!code ++]
console.warn("careful") // [!code warning]
throw new Error("boom") // [!code error]
\`\`\`
`;

    const result = await transformMarkdown(
      markdown,
      "docs/vitepress-inline.md",
      createResolvedOptions({
        codeAnnotations: {
          enabled: true,
          notation: "vitepress",
          metaKey: "annotate",
          defaultLineNumbers: false,
        },
      }),
    );

    expect(result.html).not.toContain("[!code");
    expect(result.html).toContain("has-focused");
    expect(result.html).toContain("has-diff");
    expect(result.html).toContain("ox-code-line--focus");
    expect(result.html).toContain("ox-code-line--remove");
    expect(result.html).toContain("ox-code-line--add");
    expect(result.html).toContain("ox-code-line--warning");
    expect(result.html).toContain("ox-code-line--error");
  });
});

function createResolvedOptions(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return {
    srcDir: "content",
    outDir: "dist",
    base: "/",
    ssg: {
      enabled: true,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
    },
    gfm: true,
    footnotes: true,
    tables: true,
    taskLists: true,
    strikethrough: true,
    highlight: true,
    highlightTheme: "github-dark",
    highlightLangs: [],
    codeAnnotations: {
      enabled: true,
      notation: "attribute",
      metaKey: "annotate",
      defaultLineNumbers: false,
    },
    mermaid: false,
    frontmatter: true,
    toc: true,
    tocMaxDepth: 3,
    ogImage: false,
    ogImageOptions: {
      width: 1200,
      height: 630,
      cache: true,
      concurrency: 1,
      vuePlugin: "vitejs",
    },
    transformers: [],
    docs: false,
    search: {
      enabled: true,
      limit: 10,
      prefix: true,
      placeholder: "Search documentation...",
      hotkey: "/",
    },
    ogViewer: false,
    i18n: false,
    ...overrides,
  };
}
