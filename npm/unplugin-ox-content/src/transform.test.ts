import { createRequire } from "node:module";
import type MarkdownIt from "markdown-it";
import rehypeAutolinkHeadings from "rehype-autolink-headings";
import rehypeExternalLinks from "rehype-external-links";
import rehypeKatex from "rehype-katex";
import rehypeStringify from "rehype-stringify";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import rehypeSlug from "rehype-slug";
import remarkDirective from "remark-directive";
import remarkFrontmatter from "remark-frontmatter";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import remarkRehype from "remark-rehype";
import remarkSmartypants from "remark-smartypants";
import remarkToc from "remark-toc";
import { unified } from "unified";
import { describe, expect, it } from "vite-plus/test";
import type { ResolvedOptions } from "./types";

const require = createRequire(import.meta.url);
const napiId = require.resolve("@ox-content/napi");
const MAGIC = 0x3152444d;
const VERSION = 1;
const HEADER_LEN = 28;
const NODE_RECORD_LEN = 60;
const NONE_U32 = 0xffffffff;
const TRANSFER_MAGIC = 0x5254584f;
const TRANSFER_VERSION = 1;
const TRANSFER_HEADER_LEN = 24;
const TRANSFER_SECTION_RECORD_LEN = 12;
const MDAST_SECTION_NODES = 1;
const MDAST_SECTION_CHILD_INDICES = 2;
const MDAST_SECTION_ALIGNS = 3;
const MDAST_SECTION_STRINGS = 4;
const MDAST_SECTION_CONTENT = 5;
const MDAST_SECTION_FRONTMATTER = 6;
const MDAST_SECTION_SOURCE_ORIGIN = 7;
const PREPARED_SOURCE_SECTION_CONTENT = 1;
const PREPARED_SOURCE_SECTION_FRONTMATTER = 2;
const PREPARED_SOURCE_SECTION_SOURCE_ORIGIN = 3;

const baseMdast = {
  type: "root" as const,
  children: [
    {
      type: "heading",
      depth: 1,
      children: [{ type: "text", value: "Hello" }],
    },
    {
      type: "paragraph",
      children: [{ type: "text", value: "World" }],
    },
  ],
};

function createRawMdastBuffer(): Uint8Array {
  const encoder = new TextEncoder();
  const strings = ["Hello", "World"];
  const stringOffsets = new Map<string, [number, number]>();
  let stringBytesLength = 0;

  for (const value of strings) {
    const bytes = encoder.encode(value);
    stringOffsets.set(value, [stringBytesLength, bytes.length]);
    stringBytesLength += bytes.length;
  }

  const nodeCount = 5;
  const childCount = 4;
  const alignCount = 0;
  const nodesOffset = HEADER_LEN;
  const childIndicesOffset = nodesOffset + nodeCount * NODE_RECORD_LEN;
  const stringsOffset = childIndicesOffset + childCount * 4 + alignCount;
  const buffer = new Uint8Array(stringsOffset + stringBytesLength);
  const view = new DataView(buffer.buffer);

  const writeNode = (
    index: number,
    kind: number,
    childStart: number,
    childLen: number,
    num0 = 0,
    num1 = 0,
    str0?: string,
  ) => {
    const base = nodesOffset + index * NODE_RECORD_LEN;
    const [strOffset, strLen] = str0 ? stringOffsets.get(str0)! : [NONE_U32, 0];
    view.setUint8(base, kind);
    view.setUint8(base + 1, 0);
    view.setUint16(base + 2, 0, true);
    view.setUint32(base + 4, 0, true);
    view.setUint32(base + 8, 0, true);
    view.setUint32(base + 12, childStart, true);
    view.setUint32(base + 16, childLen, true);
    view.setUint32(base + 20, num0, true);
    view.setUint32(base + 24, num1, true);
    view.setUint32(base + 28, strOffset, true);
    view.setUint32(base + 32, strLen, true);
    view.setUint32(base + 36, NONE_U32, true);
    view.setUint32(base + 40, 0, true);
    view.setUint32(base + 44, NONE_U32, true);
    view.setUint32(base + 48, 0, true);
    view.setUint32(base + 52, NONE_U32, true);
    view.setUint32(base + 56, 0, true);
  };

  view.setUint32(0, MAGIC, true);
  view.setUint32(4, VERSION, true);
  view.setUint32(8, nodeCount, true);
  view.setUint32(12, childCount, true);
  view.setUint32(16, alignCount, true);
  view.setUint32(20, stringBytesLength, true);
  view.setUint32(24, 4, true);

  writeNode(0, 12, 0, 0, 0, 0, "Hello");
  writeNode(1, 2, 0, 1, 1);
  writeNode(2, 12, 0, 0, 0, 0, "World");
  writeNode(3, 1, 1, 1);
  writeNode(4, 0, 2, 2);

  view.setUint32(childIndicesOffset, 0, true);
  view.setUint32(childIndicesOffset + 4, 2, true);
  view.setUint32(childIndicesOffset + 8, 1, true);
  view.setUint32(childIndicesOffset + 12, 3, true);

  for (const value of strings) {
    const [offset] = stringOffsets.get(value)!;
    buffer.set(encoder.encode(value), stringsOffset + offset);
  }

  return buffer;
}

function createMdastTransformBuffer(
  content: string,
  frontmatter: Record<string, unknown>,
  sourceOffset = { byteOffset: 0, offset: 0, line: 1, column: 1 },
): Uint8Array {
  const encoder = new TextEncoder();
  const legacy = createRawMdastBuffer();
  const legacyView = new DataView(legacy.buffer, legacy.byteOffset, legacy.byteLength);
  const nodeCount = legacyView.getUint32(8, true);
  const childCount = legacyView.getUint32(12, true);
  const alignCount = legacyView.getUint32(16, true);
  const stringBytesLength = legacyView.getUint32(20, true);
  const rootIndex = legacyView.getUint32(24, true);

  const nodesOffset = HEADER_LEN;
  const childIndicesOffset = nodesOffset + nodeCount * NODE_RECORD_LEN;
  const alignsOffset = childIndicesOffset + childCount * 4;
  const stringsOffset = alignsOffset + alignCount;

  const sections = [
    { id: MDAST_SECTION_NODES, bytes: legacy.slice(nodesOffset, childIndicesOffset) },
    { id: MDAST_SECTION_CHILD_INDICES, bytes: legacy.slice(childIndicesOffset, alignsOffset) },
    { id: MDAST_SECTION_ALIGNS, bytes: legacy.slice(alignsOffset, stringsOffset) },
    {
      id: MDAST_SECTION_STRINGS,
      bytes: legacy.slice(stringsOffset, stringsOffset + stringBytesLength),
    },
    { id: MDAST_SECTION_CONTENT, bytes: encoder.encode(content) },
    {
      id: MDAST_SECTION_FRONTMATTER,
      bytes: encoder.encode(JSON.stringify(frontmatter)),
    },
    { id: MDAST_SECTION_SOURCE_ORIGIN, bytes: createSourceOriginBytes(sourceOffset) },
  ];
  return createTransferEnvelope(1, sections, rootIndex);
}

function createPreparedSourceBuffer(
  content: string,
  frontmatter: Record<string, unknown>,
  sourceOffset = { byteOffset: 0, offset: 0, line: 1, column: 1 },
): Uint8Array {
  const encoder = new TextEncoder();

  return createTransferEnvelope(3, [
    { id: PREPARED_SOURCE_SECTION_CONTENT, bytes: encoder.encode(content) },
    {
      id: PREPARED_SOURCE_SECTION_FRONTMATTER,
      bytes: encoder.encode(JSON.stringify(frontmatter)),
    },
    { id: PREPARED_SOURCE_SECTION_SOURCE_ORIGIN, bytes: createSourceOriginBytes(sourceOffset) },
  ]);
}

function createSourceOriginBytes(sourceOffset: {
  byteOffset: number;
  offset: number;
  line: number;
  column: number;
}): Uint8Array {
  const buffer = new Uint8Array(16);
  const view = new DataView(buffer.buffer);
  view.setUint32(0, sourceOffset.byteOffset, true);
  view.setUint32(4, sourceOffset.offset, true);
  view.setUint32(8, sourceOffset.line, true);
  view.setUint32(12, sourceOffset.column, true);
  return buffer;
}

function createTransferEnvelope(
  kind: number,
  sections: Array<{ id: number; bytes: Uint8Array }>,
  rootHandle = 0,
): Uint8Array {
  const totalLen =
    TRANSFER_HEADER_LEN +
    sections.length * TRANSFER_SECTION_RECORD_LEN +
    sections.reduce((sum, section) => sum + section.bytes.length, 0);
  const buffer = new Uint8Array(totalLen);
  const view = new DataView(buffer.buffer);

  view.setUint32(0, TRANSFER_MAGIC, true);
  view.setUint16(4, TRANSFER_VERSION, true);
  view.setUint16(6, kind, true);
  view.setUint32(8, 1, true);
  view.setUint32(12, sections.length, true);
  view.setUint32(16, rootHandle, true);
  view.setUint32(20, 0, true);

  let sectionOffset = TRANSFER_HEADER_LEN + sections.length * TRANSFER_SECTION_RECORD_LEN;
  let recordOffset = TRANSFER_HEADER_LEN;
  for (const section of sections) {
    view.setUint32(recordOffset, section.id, true);
    view.setUint32(recordOffset + 4, sectionOffset, true);
    view.setUint32(recordOffset + 8, section.bytes.length, true);
    buffer.set(section.bytes, sectionOffset);
    sectionOffset += section.bytes.length;
    recordOffset += TRANSFER_SECTION_RECORD_LEN;
  }

  return buffer;
}

function splitFrontmatterForMock(
  source: string,
  enabled: boolean,
): {
  content: string;
  frontmatter: Record<string, unknown>;
  sourceOffset: { byteOffset: number; offset: number; line: number; column: number };
} {
  if (!enabled || !source.startsWith("---")) {
    return {
      content: source,
      frontmatter: {},
      sourceOffset: { byteOffset: 0, offset: 0, line: 1, column: 1 },
    };
  }

  const rest = source.slice(3);
  const endOffset = rest.indexOf("\n---");
  if (endOffset === -1) {
    return {
      content: source,
      frontmatter: {},
      sourceOffset: { byteOffset: 0, offset: 0, line: 1, column: 1 },
    };
  }

  const content = rest.slice(endOffset + 4).replace(/^\n/, "");
  return {
    content,
    frontmatter: { title: "frontmatter-from-rust" },
    sourceOffset: computeSourceOffsetForMock(source, content),
  };
}

function computeSourceOffsetForMock(source: string, content: string) {
  const prefix = source.slice(0, Math.max(0, source.length - content.length));
  let byteOffset = 0;
  let offset = 0;
  let line = 1;
  let column = 1;

  for (const character of prefix) {
    const codePoint = character.codePointAt(0) ?? 0;
    byteOffset += codePoint <= 0x7f ? 1 : codePoint <= 0x7ff ? 2 : codePoint <= 0xffff ? 3 : 4;
    offset += character.length;

    if (character === "\n") {
      line += 1;
      column = 1;
    } else {
      column += 1;
    }
  }

  return { byteOffset, offset, line, column };
}

require.cache[napiId] = {
  exports: {
    prepareSourceRaw: (source: string, options?: { frontmatter?: boolean }) => {
      const parsed = splitFrontmatterForMock(source, options?.frontmatter !== false);
      return createPreparedSourceBuffer(parsed.content, parsed.frontmatter, parsed.sourceOffset);
    },
    parseTransferRaw: () => createRawMdastBuffer(),
    parseMdastRaw: () => createRawMdastBuffer(),
    transformMdastRaw: (source: string, options?: { frontmatter?: boolean }) => {
      const parsed = splitFrontmatterForMock(source, options?.frontmatter !== false);
      return createMdastTransformBuffer(parsed.content, parsed.frontmatter, parsed.sourceOffset);
    },
    transform: (source: string, options?: { frontmatter?: boolean }) => ({
      html: "<h1>Hello</h1>\n<p>World</p>\n",
      frontmatter:
        options?.frontmatter === false || !source.startsWith("---")
          ? "{}"
          : JSON.stringify({ title: "frontmatter-from-rust" }),
      toc: [{ depth: 1, text: "Hello", slug: "hello" }],
      errors: [],
    }),
  },
} as never;

const { defineMdastPlugin, oxContentMdast } = await import("./mdast");
const { transformMarkdown } = await import("./transform");

function createResolvedOptions(overrides?: Partial<ResolvedOptions>): ResolvedOptions {
  return {
    srcDir: "docs",
    gfm: true,
    footnotes: true,
    tables: true,
    taskLists: true,
    strikethrough: true,
    highlight: false,
    highlightTheme: "github-dark",
    mermaid: false,
    frontmatter: true,
    toc: true,
    tocMaxDepth: 3,
    extensions: [".md"],
    include: [],
    exclude: [],
    plugin: {
      oxContent: [],
      markdownIt: [],
      mdast: [],
      remark: [],
      rehype: [],
    },
    docs: {
      enabled: false,
      src: ["./src"],
      out: "docs/api",
      include: ["**/*.ts"],
      exclude: ["**/*.test.ts"],
      includePrivate: false,
      toc: true,
      groupBy: "file",
    },
    ...overrides,
  };
}

describe("mdast js plugin", () => {
  it("keeps the native fast path when no unified plugins are configured", async () => {
    const result = await transformMarkdown("# Hello", "docs/fast-path.md", createResolvedOptions());

    expect(result.html).toContain("<h1>Hello</h1>");
    expect(result.toc).toEqual([
      {
        depth: 1,
        text: "Hello",
        slug: "hello",
        children: [],
      },
    ]);
  });

  it("passes frontmatter=false through to the Rust fast path", async () => {
    const result = await transformMarkdown(
      "---\ntitle: Ignored\n---\n# Hello",
      "docs/frontmatter-disabled.md",
      createResolvedOptions({
        frontmatter: false,
      }),
    );

    expect(result.frontmatter).toEqual({});
  });

  it("rebases native mdast positions to the original source after frontmatter", async () => {
    const result = await transformMarkdown(
      "---\ntitle: Example\n---\n# Hello",
      "docs/frontmatter-positions.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [
            defineMdastPlugin("annotate-position", (tree) => {
              const heading = tree.children[0];
              const text = heading.children?.[0];
              if (text && typeof text.value === "string") {
                const start = heading.position?.start;
                text.value = `${start?.line}:${start?.column}:${start?.offset}`;
              }
            }),
          ],
          remark: [],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<h1>4:1:23</h1>");
  });

  it("exposes source offsets on the mdast plugin context", async () => {
    const result = await transformMarkdown(
      "---\ntitle: Example\n---\n# Hello",
      "docs/mdast-context-source-offset.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [
            defineMdastPlugin("annotate-context-offset", (tree, context) => {
              const heading = tree.children[0];
              const text = heading.children?.[0];
              if (text && typeof text.value === "string") {
                const origin = context.sourceOffset;
                text.value = `${origin?.line}:${origin?.column}:${origin?.offset}`;
              }
            }),
          ],
          remark: [],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<h1>4:1:23</h1>");
  });

  it("exposes source offsets from Rust-prepared source to markdown-it plugins", async () => {
    const result = await transformMarkdown(
      "---\ntitle: Example\n---\n# Hello",
      "docs/markdown-it-source-offset.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [
            (markdownIt) => {
              markdownIt.core.ruler.push("annotate-source-offset", (state) => {
                const inline = state.tokens[1];
                const origin = state.env.oxContent?.sourceOffset;
                if (inline?.children?.[0] && origin) {
                  inline.children[0].content = `${origin.line}:${origin.column}:${origin.offset}`;
                }
              });
            },
          ],
          mdast: [],
          remark: [],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<h1>4:1:23</h1>");
  });

  it("exposes source offsets on top-level file.data for unified plugins", async () => {
    function remarkReadSourceOffset() {
      return (
        tree: { children?: Array<{ children?: Array<{ value?: string }> }> },
        file: never,
      ) => {
        const sourceOffset = (
          file as {
            data?: {
              sourceOffset?: {
                line: number;
                column: number;
                offset: number;
              };
            };
          }
        ).data?.sourceOffset;
        const text = tree.children?.[0]?.children?.[0];
        if (text && typeof text.value === "string" && sourceOffset) {
          text.value = `${sourceOffset.line}:${sourceOffset.column}:${sourceOffset.offset}`;
        }
      };
    }

    const result = await transformMarkdown(
      "---\ntitle: Example\n---\n# Hello",
      "docs/file-data-source-offset.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [remarkReadSourceOffset],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<h1>4:1:23</h1>");
  });

  it("runs Ox Content-native mdast plugins and updates the TOC from the transformed tree", async () => {
    const result = await transformMarkdown(
      "# Hello",
      "docs/mdast-plugin.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [
            defineMdastPlugin("shout-heading", (tree) => {
              const heading = tree.children[0];
              const text = heading.children?.[0];
              if (text && typeof text.value === "string") {
                text.value = `${text.value}!!!`;
              }
            }),
          ],
          remark: [],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<h1>Hello!!!</h1>");
    expect(result.toc).toEqual([
      {
        depth: 1,
        text: "Hello!!!",
        slug: "hello",
        children: [],
      },
    ]);
  });

  it("runs existing unified remark plugins in the mdast stage", async () => {
    function remarkAppendParagraph() {
      return (tree: typeof baseMdast) => {
        tree.children.push({
          type: "paragraph",
          children: [{ type: "text", value: "From remark plugin" }],
        });
      };
    }

    const result = await transformMarkdown(
      "# Hello",
      "docs/remark-plugin.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [remarkAppendParagraph],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<p>From remark plugin</p>");
  });

  it("accepts unified preset objects in the remark stage", async () => {
    function remarkAppendParagraph() {
      return (tree: typeof baseMdast) => {
        tree.children.push({
          type: "paragraph",
          children: [{ type: "text", value: "From remark preset" }],
        });
      };
    }

    const result = await transformMarkdown(
      "# Hello",
      "docs/remark-preset.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [
            {
              plugins: [remarkAppendParagraph],
            },
          ],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<p>From remark preset</p>");
  });

  it("exposes Rust-parsed frontmatter on vfile data for native unified plugins", async () => {
    function remarkReadFrontmatter() {
      return (tree: typeof baseMdast, file: { data?: { matter?: { title?: string } } }) => {
        tree.children = [
          {
            type: "heading",
            depth: 1,
            children: [
              {
                type: "text",
                value: file.data?.matter?.title ?? "missing-frontmatter",
              },
            ],
          },
        ];
      };
    }

    const result = await transformMarkdown(
      "---\ntitle: Frontmatter Title\n---\n# Ignored",
      "docs/frontmatter-data.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [remarkReadFrontmatter],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<h1>frontmatter-from-rust</h1>");
  });

  it("fails fast when modern Rust transfer bindings are unavailable", async () => {
    const napiExports = require.cache[napiId]?.exports as {
      prepareSourceRaw?: unknown;
      transformMdastRaw?: unknown;
    };
    const originalPrepareSourceRaw = napiExports.prepareSourceRaw;
    const originalTransformMdastRaw = napiExports.transformMdastRaw;
    napiExports.prepareSourceRaw = undefined;
    napiExports.transformMdastRaw = undefined;

    function remarkForceRemarkParse(this: { data: (key: string, value?: unknown) => unknown }) {
      this.data("micromarkExtensions", []);
      this.data("fromMarkdownExtensions", []);
    }

    function remarkReadFrontmatter() {
      return (tree: typeof baseMdast, file: { data?: { matter?: { title?: string } } }) => {
        tree.children = [
          {
            type: "heading",
            depth: 1,
            children: [
              {
                type: "text",
                value: file.data?.matter?.title ?? "missing-frontmatter",
              },
            ],
          },
        ];
      };
    }

    try {
      await expect(
        transformMarkdown(
          "---\ntitle: Frontmatter Title\n---\n# Ignored",
          "docs/frontmatter-fallback.md",
          createResolvedOptions({
            plugin: {
              oxContent: [],
              markdownIt: [],
              mdast: [],
              remark: [remarkForceRemarkParse, remarkReadFrontmatter],
              rehype: [],
            },
          }),
        ),
      ).rejects.toThrow(/missing prepareSourceRaw/);
    } finally {
      napiExports.prepareSourceRaw = originalPrepareSourceRaw;
      napiExports.transformMdastRaw = originalTransformMdastRaw;
    }
  });

  it("fails fast when native mdast transfer bindings are unavailable", async () => {
    const napiExports = require.cache[napiId]?.exports as {
      transformMdastRaw?: unknown;
    };
    const originalTransformMdastRaw = napiExports.transformMdastRaw;
    napiExports.transformMdastRaw = undefined;

    try {
      await expect(
        transformMarkdown(
          "# Hello",
          "docs/native-mdast-missing.md",
          createResolvedOptions({
            plugin: {
              oxContent: [],
              markdownIt: [],
              mdast: [
                defineMdastPlugin("noop", () => {
                  return;
                }),
              ],
              remark: [],
              rehype: [],
            },
          }),
        ),
      ).rejects.toThrow(/missing transformMdastRaw/);
    } finally {
      napiExports.transformMdastRaw = originalTransformMdastRaw;
    }
  });

  it("exposes Rust-prepared frontmatter to markdown-it plugins", async () => {
    function markdownItFrontmatterPlugin(md: MarkdownIt) {
      md.core.ruler.push("frontmatter-heading", (state) => {
        const inline = state.tokens[1];
        if (!inline || inline.type !== "inline") {
          return;
        }

        for (const child of inline.children ?? []) {
          if (child.type === "text") {
            child.content = String(
              (state.env as { frontmatter?: { title?: string } }).frontmatter?.title,
            );
          }
        }
      });
    }

    const result = await transformMarkdown(
      "---\ntitle: Frontmatter Title\n---\n# Ignored",
      "docs/markdown-it-frontmatter.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [markdownItFrontmatterPlugin],
          mdast: [],
          remark: [],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<h1>frontmatter-from-rust</h1>");
    expect(result.frontmatter).toEqual({ title: "frontmatter-from-rust" });
  });

  it("falls back to remark-parse when remark syntax extensions are registered", async () => {
    function remarkForceRemarkParse(this: { data: (key: string, value?: unknown) => unknown }) {
      this.data("micromarkExtensions", []);
      this.data("fromMarkdownExtensions", []);
    }

    function remarkAnnotateHeading() {
      return (tree: typeof baseMdast) => {
        const heading = tree.children[0];
        const text = heading.children?.[0];
        if (text && typeof text.value === "string") {
          text.value = `${text.value} via remark-parse`;
        }
      };
    }

    const result = await transformMarkdown(
      "# Parsed by fallback",
      "docs/remark-fallback.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [remarkForceRemarkParse, remarkAnnotateHeading],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<h1>Parsed by fallback via remark-parse</h1>");
    expect(result.html).not.toContain("<h1>Hello</h1>");
  });

  it("honors custom unified parsers without overriding them", async () => {
    function useCustomParser(this: { parser?: (document: string) => typeof baseMdast }) {
      this.parser = (document: string) => ({
        type: "root",
        children: [
          {
            type: "heading",
            depth: 1,
            children: [
              {
                type: "text",
                value: document.startsWith("---") ? "frontmatter-visible" : "frontmatter-hidden",
              },
            ],
          },
        ],
      });
    }

    const result = await transformMarkdown(
      "---\ntitle: Example\n---\n# Ignored by custom parser",
      "docs/custom-parser.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [useCustomParser],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<h1>frontmatter-visible</h1>");
    expect(result.html).not.toContain("<h1>Hello</h1>");
  });

  it("reuses explicit remark-rehype and rehype-stringify plugins without double-applying them", async () => {
    const result = await transformMarkdown(
      "# Hello",
      "docs/explicit-bridge.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [[remarkRehype, { allowDangerousHtml: true }]],
          rehype: [[rehypeStringify, { allowDangerousHtml: true }]],
        },
      }),
    );

    expect(result.html).toContain("<h1>Hello</h1>");
    expect(result.html).toContain("<p>World</p>");
  });

  it("reuses explicit bridge plugins nested inside unified presets", async () => {
    function rehypeAnnotateHeading() {
      return (tree: {
        children?: Array<{
          type?: string;
          tagName?: string;
          properties?: Record<string, unknown>;
        }>;
      }) => {
        const heading = tree.children?.find(
          (node) => node.type === "element" && node.tagName === "h1",
        );
        if (!heading) {
          return;
        }

        heading.properties = {
          ...heading.properties,
          dataPreset: "bridge",
        };
      };
    }

    const result = await transformMarkdown(
      "# Hello",
      "docs/preset-bridge.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [
            {
              plugins: [[remarkRehype, { allowDangerousHtml: true }]],
            },
          ],
          rehype: [
            {
              plugins: [rehypeAnnotateHeading, [rehypeStringify, { allowDangerousHtml: true }]],
            },
          ],
        },
      }),
    );

    expect(result.html).toContain('<h1 data-preset="bridge">Hello</h1>');
    expect(result.html).toContain("<p>World</p>");
  });

  it("honors custom unified compilers without overriding them", async () => {
    function useCustomCompiler(this: { compiler?: (tree: typeof baseMdast) => string }) {
      this.compiler = (tree) => {
        const heading = tree.children[0];
        const text = heading.children?.[0];
        return `<section data-compiler="remark">${typeof text?.value === "string" ? text.value : "missing"}</section>`;
      };
    }

    const result = await transformMarkdown(
      "# Hello",
      "docs/custom-compiler.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [useCustomCompiler],
          rehype: [],
        },
      }),
    );

    expect(result.html).toBe('<section data-compiler="remark">Hello</section>');
  });

  it("honors custom rehype compilers without overriding them", async () => {
    function rehypeCustomCompiler(this: { compiler?: () => string }) {
      this.compiler = () => '<aside data-compiler="rehype">custom-output</aside>';
    }

    const result = await transformMarkdown(
      "# Hello",
      "docs/rehype-custom-compiler.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [],
          rehype: [rehypeCustomCompiler],
        },
      }),
    );

    expect(result.html).toBe('<aside data-compiler="rehype">custom-output</aside>');
  });

  it("runs a real-world remark syntax and transform matrix end-to-end", async () => {
    const result = await transformMarkdown(
      [
        "# Guide",
        "",
        "## Contents",
        "",
        "## Install",
        "",
        'Run "vp install" -- then enjoy.',
        "",
        "- [x] shipped",
        "",
        "| Package | Runtime |",
        "| - | - |",
        "| ox-content | Node |",
        "",
        "~~old~~",
      ].join("\n"),
      "docs/real-remark-matrix.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [remarkGfm, remarkSmartypants, [remarkToc, { heading: "contents" }]],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<table>");
    expect(result.html).toMatch(/<input(?=[^>]*type="checkbox")(?=[^>]*checked)[^>]*>/);
    expect(result.html).toContain("<del>old</del>");
    expect(result.html).toContain('href="#install"');
    expect(result.html).toMatch(/“vp install”|&#x201C;vp install&#x201D;/);
  });

  it("runs remark-frontmatter while preserving Rust-prepared vfile matter", async () => {
    function remarkReportFrontmatterNodes() {
      return (
        tree: {
          children?: Array<{
            type?: string;
            value?: string;
          }>;
        },
        file: { data?: { matter?: { title?: string } } },
      ) => {
        const yaml = tree.children?.find((node) => node.type === "yaml");
        tree.children = (tree.children ?? []).filter((node) => node.type !== "yaml");
        tree.children.push({
          type: "paragraph",
          children: [
            {
              type: "text",
              value:
                `frontmatter-node:${yaml?.value?.trim() ?? "missing"}; ` +
                `matter:${file.data?.matter?.title ?? "missing"}`,
            },
          ],
        } as never);
      };
    }

    const result = await transformMarkdown(
      "---\ntitle: Plugin Frontmatter\n---\n# Body",
      "docs/real-frontmatter.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [[remarkFrontmatter, ["yaml"]], remarkReportFrontmatterNodes],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain(
      "<p>frontmatter-node:title: Plugin Frontmatter; matter:frontmatter-from-rust</p>",
    );
  });

  it("runs remark-directive syntax with downstream mdast-to-hast data", async () => {
    type DirectiveNode = {
      type?: string;
      name?: string;
      attributes?: Record<string, string>;
      data?: Record<string, unknown>;
      children?: DirectiveNode[];
    };

    function remarkDirectiveToAside() {
      return (tree: DirectiveNode) => {
        const visit = (node: DirectiveNode) => {
          if (node.type === "containerDirective" && node.name === "note") {
            node.data = {
              ...node.data,
              hName: "aside",
              hProperties: {
                id: node.attributes?.id,
                className: ["note"],
              },
            };
          }

          for (const child of node.children ?? []) {
            visit(child);
          }
        };

        visit(tree);
      };
    }

    const result = await transformMarkdown(
      ":::note{#plugin-note}\nDirective **body**.\n:::",
      "docs/real-directive.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [remarkDirective, remarkDirectiveToAside],
          rehype: [],
        },
      }),
    );

    expect(result.html).toMatch(/<aside(?=[^>]*id="plugin-note")(?=[^>]*class="note")[^>]*>/);
    expect(result.html).toContain("Directive <strong>body</strong>.");
  });

  it("runs remark-math with rehype-katex end-to-end", async () => {
    const result = await transformMarkdown(
      "Inline $a + b$.\n\n$$\nx^2\n$$",
      "docs/real-math.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [remarkMath],
          rehype: [rehypeKatex],
        },
      }),
    );

    expect(result.html).toContain('class="katex"');
    expect(result.html).toContain("a");
    expect(result.html).toContain("x");
  });

  it("runs rehype-slug and rehype-autolink-headings on the native mdast bridge", async () => {
    const result = await transformMarkdown(
      "# Ignored by native mock",
      "docs/real-rehype-headings.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [],
          rehype: [
            rehypeSlug,
            [
              rehypeAutolinkHeadings,
              {
                behavior: "append",
                content: {
                  type: "text",
                  value: "#",
                },
              },
            ],
          ],
        },
      }),
    );

    expect(result.html).toContain('<h1 id="hello">Hello');
    expect(result.html).toContain('href="#hello"');
    expect(result.html).toContain(">#</a>");
  });

  it("runs rehype-external-links after remark fallback parsing", async () => {
    const result = await transformMarkdown(
      "[External](https://example.com) and [Local](/docs)",
      "docs/real-external-links.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [remarkGfm],
          rehype: [
            [
              rehypeExternalLinks,
              {
                target: "_blank",
                rel: ["nofollow", "noopener"],
              },
            ],
          ],
        },
      }),
    );

    expect(result.html).toMatch(
      /<a(?=[^>]*href="https:\/\/example\.com")(?=[^>]*target="_blank")(?=[^>]*rel="nofollow noopener")[^>]*>External<\/a>/,
    );
    expect(result.html).toContain('<a href="/docs">Local</a>');
  });

  it("runs rehype-raw and rehype-sanitize for raw HTML compatibility", async () => {
    const result = await transformMarkdown(
      "Safe <b>bold</b>.\n\n<script>alert(1)</script>",
      "docs/real-raw-sanitize.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [],
          mdast: [],
          remark: [remarkGfm],
          rehype: [rehypeRaw, rehypeSanitize],
        },
      }),
    );

    expect(result.html).toContain("<b>bold</b>");
    expect(result.html).not.toContain("<script>");
    expect(result.html).not.toContain("alert(1)");
  });

  it("runs markdown-it plugins and builds the TOC from markdown-it tokens", async () => {
    function markdownItHeadingPlugin(md: MarkdownIt) {
      md.core.ruler.push("rewrite-heading", (state) => {
        for (let index = 0; index < state.tokens.length; index += 1) {
          const token = state.tokens[index];
          if (token.type !== "heading_open") {
            continue;
          }

          const inline = state.tokens[index + 1];
          if (!inline || inline.type !== "inline") {
            continue;
          }

          token.attrSet("id", "markdown-it-heading");
          for (const child of inline.children ?? []) {
            if (child.type === "text") {
              child.content = child.content.replace("Hello", "Hello from markdown-it");
            }
          }
        }
      });
    }

    const result = await transformMarkdown(
      "# Hello",
      "docs/markdown-it-plugin.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [markdownItHeadingPlugin],
          mdast: [],
          remark: [],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain('<h1 id="markdown-it-heading">Hello from markdown-it</h1>');
    expect(result.toc).toEqual([
      {
        depth: 1,
        text: "Hello from markdown-it",
        slug: "markdown-it-heading",
        children: [],
      },
    ]);
  });

  it("pipes markdown-it output through rehype plugins when configured", async () => {
    function markdownItHeadingPlugin(md: MarkdownIt) {
      md.core.ruler.push("rewrite-heading", (state) => {
        const inline = state.tokens[1];
        if (!inline || inline.type !== "inline") {
          return;
        }

        for (const child of inline.children ?? []) {
          if (child.type === "text") {
            child.content = "Hello from markdown-it";
          }
        }
      });
    }

    function rehypeAnnotateHeading() {
      return (tree: {
        children?: Array<{
          type?: string;
          tagName?: string;
          properties?: Record<string, unknown>;
        }>;
      }) => {
        const heading = tree.children?.find(
          (node) => node.type === "element" && node.tagName === "h1",
        );
        if (!heading) {
          return;
        }

        heading.properties = {
          ...heading.properties,
          dataPipeline: "rehype",
        };
      };
    }

    const result = await transformMarkdown(
      "# Hello",
      "docs/markdown-it-rehype.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [markdownItHeadingPlugin],
          mdast: [],
          remark: [],
          rehype: [rehypeAnnotateHeading],
        },
      }),
    );

    expect(result.html).toContain('data-pipeline="rehype"');
    expect(result.html).toContain("Hello from markdown-it");
  });

  it("bridges markdown-it output into mdast and remark plugins when both are configured", async () => {
    function markdownItHeadingPlugin(md: MarkdownIt) {
      md.core.ruler.push("rewrite-heading", (state) => {
        const inline = state.tokens[1];
        if (!inline || inline.type !== "inline") {
          return;
        }

        for (const child of inline.children ?? []) {
          if (child.type === "text") {
            child.content = "Hello from markdown-it";
          }
        }
      });
    }

    function remarkAppendParagraph() {
      return (tree: typeof baseMdast) => {
        tree.children.push({
          type: "paragraph",
          children: [{ type: "text", value: "From remark bridge" }],
        });
      };
    }

    const result = await transformMarkdown(
      "# Hello",
      "docs/mixed-pipeline.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [markdownItHeadingPlugin],
          mdast: [
            defineMdastPlugin("annotate-heading", (tree) => {
              const heading = tree.children[0];
              const text = heading.children?.[0];
              if (text && typeof text.value === "string") {
                text.value = `${text.value} + mdast`;
              }
            }),
          ],
          remark: [remarkAppendParagraph],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<h1>Hello from markdown-it + mdast</h1>");
    expect(result.html).toContain("<p>From remark bridge</p>");
    expect(result.toc).toEqual([
      {
        depth: 1,
        text: "Hello from markdown-it + mdast",
        slug: "hello-from-markdown-it-mdast",
        children: [],
      },
    ]);
  });

  it("exposes markdown-it tokens on vfile data for downstream unified plugins", async () => {
    function markdownItHeadingPlugin(md: MarkdownIt) {
      md.core.ruler.push("rewrite-heading", (state) => {
        const inline = state.tokens[1];
        if (!inline || inline.type !== "inline") {
          return;
        }

        for (const child of inline.children ?? []) {
          if (child.type === "text") {
            child.content = "Hello from markdown-it tokens";
          }
        }
      });
    }

    function remarkReadMarkdownItTokens() {
      return (
        tree: typeof baseMdast,
        file: {
          data?: {
            oxContent?: {
              markdownIt?: {
                tokens?: Array<{
                  type?: string;
                  children?: Array<{ type?: string; content?: string }>;
                }>;
              };
            };
          };
        },
      ) => {
        const inline = file.data?.oxContent?.markdownIt?.tokens?.find(
          (token) => token.type === "inline",
        );
        const text = inline?.children?.find((token) => token.type === "text")?.content;
        if (!text) {
          return;
        }

        tree.children.push({
          type: "paragraph",
          children: [{ type: "text", value: `From token stream: ${text}` }],
        });
      };
    }

    const result = await transformMarkdown(
      "# Hello",
      "docs/markdown-it-vfile-data.md",
      createResolvedOptions({
        plugin: {
          oxContent: [],
          markdownIt: [markdownItHeadingPlugin],
          mdast: [],
          remark: [remarkReadMarkdownItTokens],
          rehype: [],
        },
      }),
    );

    expect(result.html).toContain("<p>From token stream: Hello from markdown-it tokens</p>");
  });

  it("can be used directly as a unified parser plugin", async () => {
    const file = await unified()
      .use(oxContentMdast, { gfm: true })
      .use(remarkRehype, { allowDangerousHtml: true })
      .use(rehypeStringify, { allowDangerousHtml: true })
      .process("# Ignored by mock parser");

    expect(String(file)).toContain("<h1>Hello</h1>");
    expect(String(file)).toContain("<p>World</p>");
  });
});
