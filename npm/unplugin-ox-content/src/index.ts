/**
 * @ox-content/unplugin
 *
 * Universal plugin for Ox Content - Markdown processing for
 * webpack, rollup, esbuild, vite, and more.
 */

import { createUnplugin, type UnpluginFactory } from "unplugin";
import { createFilter } from "@rollup/pluginutils";
import { transformMarkdown } from "./transform";
import type { OxContentOptions, ResolvedOptions, ResolvedDocsConfig, DocsConfig } from "./types";

export type {
  OxContentOptions,
  ResolvedOptions,
  ResolvedDocsConfig,
  DocsConfig,
  TocEntry,
  TransformResult,
  PluginConfig,
  OxContentPlugin,
  MarkdownItPlugin,
  RemarkPlugin,
  RehypePlugin,
} from "./types";
export { transformMarkdown } from "./transform";

/**
 * Resolves docs configuration.
 */
function resolveDocsConfig(docs: boolean | DocsConfig | undefined): ResolvedDocsConfig {
  if (docs === false || docs === undefined) {
    return {
      enabled: false,
      src: ["./src"],
      out: "docs/api",
      include: ["**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx"],
      exclude: ["**/*.test.*", "**/*.spec.*", "**/node_modules/**"],
      includePrivate: false,
      toc: true,
      groupBy: "file",
    };
  }

  if (docs === true) {
    return {
      enabled: true,
      src: ["./src"],
      out: "docs/api",
      include: ["**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx"],
      exclude: ["**/*.test.*", "**/*.spec.*", "**/node_modules/**"],
      includePrivate: false,
      toc: true,
      groupBy: "file",
    };
  }

  return {
    enabled: docs.enabled ?? true,
    src: docs.src ?? ["./src"],
    out: docs.out ?? "docs/api",
    include: docs.include ?? ["**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx"],
    exclude: docs.exclude ?? ["**/*.test.*", "**/*.spec.*", "**/node_modules/**"],
    includePrivate: docs.includePrivate ?? false,
    toc: docs.toc ?? true,
    groupBy: docs.groupBy ?? "file",
  };
}

/**
 * Resolves plugin options with defaults.
 */
function resolveOptions(options: OxContentOptions): ResolvedOptions {
  const extensions = options.extensions ?? [".md", ".markdown"];
  return {
    srcDir: options.srcDir ?? "docs",
    gfm: options.gfm ?? true,
    footnotes: options.footnotes ?? true,
    tables: options.tables ?? true,
    taskLists: options.taskLists ?? true,
    strikethrough: options.strikethrough ?? true,
    highlight: options.highlight ?? false,
    highlightTheme: options.highlightTheme ?? "github-dark",
    codeAnnotations: resolveCodeAnnotationsOptions(options.codeAnnotations),
    mermaid: options.mermaid ?? false,
    frontmatter: options.frontmatter ?? true,
    toc: options.toc ?? true,
    tocMaxDepth: options.tocMaxDepth ?? 3,
    extensions,
    include: Array.isArray(options.include)
      ? options.include
      : options.include
        ? [options.include]
        : [],
    exclude: Array.isArray(options.exclude)
      ? options.exclude
      : options.exclude
        ? [options.exclude]
        : [],
    plugin: {
      oxContent: options.plugin?.oxContent ?? [],
      markdownIt: options.plugin?.markdownIt ?? [],
      remark: options.plugin?.remark ?? [],
      rehype: options.plugin?.rehype ?? [],
    },
    docs: resolveDocsConfig(options.docs),
  };
}

function resolveCodeAnnotationsOptions(
  options: OxContentOptions["codeAnnotations"],
): ResolvedOptions["codeAnnotations"] {
  if (!options) {
    return {
      enabled: false,
      metaKey: "annotate",
    };
  }

  if (options === true) {
    return {
      enabled: true,
      metaKey: "annotate",
    };
  }

  return {
    enabled: true,
    metaKey: options.metaKey ?? "annotate",
  };
}

/**
 * Check if the file should be processed.
 */
function isMarkdownFile(id: string, options: ResolvedOptions): boolean {
  return options.extensions.some((ext) => id.endsWith(ext));
}

/**
 * The unplugin factory function.
 */
const unpluginFactory: UnpluginFactory<OxContentOptions | undefined> = (rawOptions = {}) => {
  const options = resolveOptions(rawOptions);

  const filter = createFilter(
    options.include.length > 0 ? options.include : undefined,
    options.exclude.length > 0 ? options.exclude : undefined,
  );

  return {
    name: "@ox-content/unplugin",

    resolveId(id) {
      // Handle virtual modules
      if (id.startsWith("virtual:ox-content/")) {
        return "\0" + id;
      }
      return null;
    },

    loadInclude(id) {
      // Load virtual modules
      return id.startsWith("\0virtual:ox-content/");
    },

    load(id) {
      if (id.startsWith("\0virtual:ox-content/")) {
        const path = id.slice("\0virtual:ox-content/".length);
        if (path === "config") {
          return `export default ${JSON.stringify(options)};`;
        }
        if (path === "runtime") {
          return `
            export function useMarkdown() {
              return {
                render: (content) => content,
              };
            }
          `;
        }
        return "export default {};";
      }
      return null;
    },

    transformInclude(id) {
      return isMarkdownFile(id, options) && filter(id);
    },

    async transform(code, id) {
      if (!isMarkdownFile(id, options)) {
        return null;
      }

      if (!filter(id)) {
        return null;
      }

      const result = await transformMarkdown(code, id, options);
      return {
        code: result.code,
        map: null,
      };
    },
  };
};

/**
 * The unplugin instance.
 */
export const unplugin = /* #__PURE__ */ createUnplugin(unpluginFactory);

export default unplugin;
