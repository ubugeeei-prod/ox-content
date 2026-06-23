import type { ExtractedDocs, ResolvedOptions } from "../../src/types";

export function createDocsFixture(): ExtractedDocs[] {
  return [
    {
      file: "/repo/src/math.ts",
      entries: [
        {
          name: "RoundMode",
          kind: "type",
          description: "Controls how values are rounded before they are displayed.",
          tags: {
            since: "2.0.0",
          },
          file: "/repo/src/math.ts",
          line: 3,
          endLine: 3,
        },
        {
          name: "round",
          kind: "function",
          description:
            "Rounds a numeric value using [RoundMode].\n\nUseful when an API result needs deterministic formatting.",
          params: [
            {
              name: "value",
              type: "number",
              description: "Input value to normalize",
            },
            {
              name: "decimals",
              type: "number",
              description: "Maximum fractional precision",
            },
            {
              name: "mode",
              type: "RoundMode",
              description: "Rounding behavior to apply",
              default: '"half-up"',
            },
          ],
          returns: {
            type: "number",
            description: "Rounded numeric value",
          },
          examples: ['```ts\nround(3.14159, 2, "half-up");\nround(1.005, 2, "bankers");\n```'],
          tags: {
            since: "2.0.0",
            deprecated: "Prefer `formatNumber` when returning UI strings.",
          },
          file: "/repo/src/math.ts",
          line: 10,
          endLine: 24,
          signature:
            'export function round(value: number, decimals: number, mode: RoundMode = "half-up"): number',
        },
      ],
    },
    {
      file: "/repo/src/utils.ts",
      entries: [
        {
          name: "capitalize",
          kind: "function",
          description:
            "Capitalizes user-facing labels before they are passed to [RoundMode]-aware formatters.\n\nKeeps already-uppercase acronyms unchanged.",
          params: [
            {
              name: "value",
              type: "string",
              description: "Source label to normalize",
            },
          ],
          returns: {
            type: "string",
            description: "Normalized label with an uppercased first glyph",
          },
          examples: ['```ts\ncapitalize("docs");\ncapitalize("API");\n```'],
          tags: {
            since: "1.4.0",
          },
          file: "/repo/src/utils.ts",
          line: 4,
          endLine: 14,
          signature: "export function capitalize(value: string): string",
        },
      ],
    },
  ];
}

export function createDocsResolvedOptions(
  overrides: Partial<ResolvedOptions> = {},
): ResolvedOptions {
  return {
    srcDir: "content",
    outDir: "dist",
    base: "/",
    extensions: [".md", ".markdown", ".mdx"],
    ssg: {
      enabled: true,
      extension: ".html",
      clean: false,
      bare: false,
      siteName: "Ox Content",
      generateOgImage: false,
      lastUpdated: false,
    },
    gfm: true,
    footnotes: true,
    tables: true,
    taskLists: true,
    strikethrough: true,
    highlight: true,
    highlightTheme: "vitesse-dark",
    highlightLangs: [],
    codeAnnotations: {
      enabled: false,
      notation: "attribute",
      metaKey: "annotate",
      defaultLineNumbers: false,
    },
    wikiLinks: { enabled: false, baseUrl: "/" },
    emojiShortcodes: { enabled: false, custom: {} },
    attrs: { enabled: false },
    codeImports: { enabled: false },
    sanitize: { enabled: false },
    editThisPage: { enabled: false, branch: "main", label: "Edit this page" },
    cjkEmphasis: false,
    codeBlockLint: {
      enabled: false,
      requireLanguage: false,
      trailingSpaces: true,
      mode: "warn",
    },
    codeBlockTypecheck: {
      enabled: false,
      languages: ["ts", "tsx"],
      requireMeta: true,
      tsgoCommand: "tsgo",
      mode: "warn",
    },
    docsTests: { enabled: false, languages: ["js", "jsx", "ts", "tsx"], requireMeta: true },
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
    collections: { enabled: false, collections: {} },
    ogViewer: false,
    embeds: {
      github: {},
      openGraph: {},
      pm: false,
      spotify: false,
      stackBlitz: false,
      twitter: false,
      bluesky: false,
      webContainer: false,
    },
    i18n: false,
    ...overrides,
  };
}

export function createDocsSearchIndex() {
  const body =
    "capitalize user-facing labels roundmode formatters reference signature parameters returns examples";

  return {
    documents: [
      {
        id: "api/utils",
        title: "utils.ts",
        url: "/api/utils",
        body,
        headings: ["Overview", "Reference"],
        code: ["capitalize(value: string): string"],
      },
    ],
    index: {
      capitalize: [
        { doc_idx: 0, field: "Body", tf: 2 },
        { doc_idx: 0, field: "Code", tf: 1 },
      ],
      roundmode: [{ doc_idx: 0, field: "Body", tf: 1 }],
      utils: [{ doc_idx: 0, field: "Title", tf: 1 }],
    },
    df: {
      capitalize: 1,
      roundmode: 1,
      utils: 1,
    },
    avg_dl: body.length,
    doc_count: 1,
  };
}
