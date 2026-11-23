import { resolve, extname, dirname, isAbsolute } from "node:path";
import { readFile } from "node:fs/promises";
import { parseAst, transformWithOxc } from "vite";
import {
  classifyPublishState,
  createOxContentCustomHostPlugin,
  isMarkdownFilePath,
  normalizeMarkdownExtensions,
  resolvePublishStateOptions,
  type OxContentOptions,
} from "@ox-content/vite-plugin";
import {
  createHtmlHostCollectionDocuments,
  resolveHtmlHostComponentsGlob,
  type HtmlHostComponentsOption,
  type HtmlHostComponentsMap,
  type HtmlHostCollectionDocumentsOptions,
  type HtmlHostMarkdownMetadata,
} from "@ox-content/vite-plugin/html-host";
import type { OxContentCustomHostModule } from "@ox-content/vite-plugin/custom-host";
import type { Plugin, ResolvedConfig } from "vite";
import { compileHonoDocument } from "./document.js";
import { honoJsx } from "./html-host.js";
import type { Frontmatter } from "./context.js";

export const HONO_CLIENT_ID = "virtual:ox-content-hono/client";
const resolvedClientId = "\0" + HONO_CLIENT_ID;
const codeExtensions = new Set([".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs"]);

export interface OxContentHonoOptions {
  oxContent?: OxContentOptions;
  components?: HtmlHostComponentsOption;
  /** Values intentionally exposed to browser islands. Defaults to {}. */
  frontmatter?: (frontmatter: Frontmatter) => Frontmatter;
  /** Disable content routes and automatic build input when an existing app owns routing. */
  routes?: false;
  /** Module exporting a Hono document layout. Receives children, head, frontmatter and path. */
  layout?: string;
}

/** Full content-site integration. The low-level HTML host adapters remain available. */
export function oxContentHono(options: OxContentHonoOptions = {}): Plugin[] {
  const oxContent: OxContentOptions = {
    highlight: false,
    resources: false,
    docs: false,
    search: false,
    feeds: false,
    siteMaps: false,
    ...options.oxContent,
  };
  const markdownExtensions = normalizeMarkdownExtensions(oxContent.extensions);
  const publishState = resolvePublishStateOptions(oxContent.publishState);
  const collectionDocuments: HtmlHostCollectionDocumentsOptions = {
    oxContent,
    select: (document, context) =>
      options.routes === false ||
      context.command !== "build" ||
      !publishState.enabled ||
      classifyPublishState(document.frontmatter, publishState).output,
  };
  const documents = createHtmlHostCollectionDocuments(collectionDocuments);
  let config: ResolvedConfig;
  let resolveLayoutImport:
    | ((source: string, importer: string) => Promise<string | undefined>)
    | undefined;
  let components: HtmlHostComponentsMap = {};
  const layoutDependencies = new Set<string>();
  const importedDocuments = new Map<string, string | undefined>();
  const integration: Plugin = {
    name: "ox-content:hono",
    enforce: "pre",
    config(user) {
      if (options.routes === false) return;
      return { appType: user.appType ?? "mpa" };
    },
    configEnvironment: {
      order: "post",
      handler(name, environment) {
        if (name !== "client" || options.routes === false) return;
        const build = (environment.build ??= {});
        build.manifest = true;
        // Vite has merged the inherited and environment-specific options here.
        // Mutate the input because returning arrays from a config hook concatenates them.
        const bundler = (build.rolldownOptions ??= build.rollupOptions ?? {});
        const input = bundler.input;
        if (typeof input === "string" || Array.isArray(input)) {
          const entries = typeof input === "string" ? [input] : input;
          bundler.input = [...new Set([...entries, HONO_CLIENT_ID])];
        } else if (!Object.values(input ?? {}).includes(HONO_CLIENT_ID)) {
          let key = "ox-content-hono";
          while (input && key in input) key += "_";
          bundler.input = { ...input, [key]: HONO_CLIENT_ID };
        }
      },
    },
    async configResolved(value) {
      config = value;
      components = options.components
        ? await resolveHtmlHostComponentsGlob(options.components, value.root)
        : {};
    },
    resolveId(id) {
      return id === HONO_CLIENT_ID ? resolvedClientId : null;
    },
    buildStart() {
      if (this.environment.name === "client") {
        resolveLayoutImport = async (source, importer) => {
          const resolved = await this.resolve(source, importer);
          return resolved?.external ? undefined : resolved?.id;
        };
      }
    },
    load(id) {
      if (id !== resolvedClientId) return;
      return `import modules from 'virtual:ox-content-hono/modules';
import {initHonoIslands} from '@ox-content/vite-plugin-hono/client';
const controller = initHonoIslands({modules});
if (import.meta.hot) import.meta.hot.dispose(() => controller.destroy());`;
    },
    async transform(source, id, environment) {
      if (id.includes("?") || !isMarkdownFilePath(id, markdownExtensions)) return;
      if (!environment?.ssr)
        throw new Error(
          "Hono Markdown components are server components. Import them from your server entry, not a browser entry.",
        );
      // Vite supplies source for virtual IDs. File-backed documents are read on registry rebuild.
      if (!options.layout) importedDocuments.set(id, isAbsolute(id) ? undefined : source);
      const result = await compileHonoDocument({
        source,
        documentPath: id,
        root: config.root,
        oxContent,
        components,
        selectFrontmatter: options.frontmatter,
      });
      return { code: result.code, map: null };
    },
  };
  const host: OxContentCustomHostModule = {
    async routes(context) {
      const pages = await documents({
        root: context.root,
        command: context.mode,
        mode: context.mode === "build" ? "production" : "development",
      });
      return pages.map((document) => ({
        path: document.path === "/" ? "/" : document.path.replace(/\/?$/, "/"),
        inputPath: document.documentPath,
        dependencies: [
          document.documentPath,
          ...layoutDependencies,
          ...(options.layout ? [resolve(context.root, options.layout)] : []),
        ],
        async render(ctx) {
          // Vite can prefix virtual manifest keys with the path from root to cwd.
          const clientEntry =
            ctx.mode === "serve"
              ? "/@id/__x00__" + HONO_CLIENT_ID
              : (Object.keys(ctx.assets.clientManifest ?? {}).find(
                  (key) => key === HONO_CLIENT_ID || key.endsWith("/" + HONO_CLIENT_ID),
                ) ?? HONO_CLIENT_ID);
          // Use the same Vite SSR module graph as the islands, preserving Hono
          // Context identity instead of mixing it with bundled vite.config code.
          const { renderHonoMarkdown } = (await ctx.loadModule(
            "@ox-content/vite-plugin-hono/vite",
          )) as typeof import("./vite.js");
          const page = await ctx.markdown.render<HtmlHostMarkdownMetadata>({
            source: document.source,
            documentPath: document.documentPath,
            renderHtml(markdown) {
              return renderHonoMarkdown({
                context: markdown,
                components,
                frontmatter: options.frontmatter?.(markdown.transform.frontmatter),
                documentAssets: {
                  pageStyles: options.layout
                    ? ctx.assets.ssrStylesheets({
                        modules: [resolve(ctx.root, options.layout)],
                      }).stylesheets
                    : [],
                  clientEntries:
                    options.layout ||
                    markdown.transform.components?.length ||
                    Object.keys(components).length
                      ? [clientEntry]
                      : [],
                },
              });
            },
          });
          const head = page.metadata?.documentAssets?.headHtml ?? "";
          if (options.layout) {
            const layout = (await ctx.loadModule(resolve(ctx.root, options.layout))) as {
              default: (props: Record<string, unknown>) => unknown;
            };
            const { jsx } = (await ctx.loadModule("hono/jsx")) as typeof import("hono/jsx");
            const { raw } = (await ctx.loadModule("hono/html")) as typeof import("hono/html");
            return {
              html:
                "<!doctype html>" +
                String(
                  await jsx(layout.default, {
                    children: raw(page.html),
                    head: raw(head),
                    frontmatter: document.frontmatter,
                    path: document.path,
                  }).toString(),
                ),
              dependencies: page.dependencies,
            };
          }
          return {
            html: `<!doctype html><html><head><meta charset="utf-8">${head}</head><body><main>${page.html}</main></body></html>`,
            dependencies: page.dependencies,
          };
        },
      }));
    },
  };
  return [
    integration,
    ...honoJsx({
      oxContent,
      components: options.components,
      collectionDocuments,
      documents: async () => {
        // Discover the layout's static import graph before the browser build.
        // Component imports outside content/ must also reach the lazy registry.
        const found = new Map<string, { documentPath: string; source: string }>();
        if (!options.layout) {
          for (const [documentPath, virtualSource] of importedDocuments) {
            try {
              const source = virtualSource ?? (await readFile(documentPath, "utf8"));
              found.set(documentPath, { documentPath, source });
            } catch (error) {
              const { code } = error as NodeJS.ErrnoException;
              if (code !== "ENOENT" && code !== "ENOTDIR") throw error;
              importedDocuments.delete(documentPath);
            }
          }
        }
        const visited = new Set<string>();
        async function visit(file: string): Promise<void> {
          if (visited.has(file) || file.includes("/node_modules/")) return;
          const markdown = isMarkdownFilePath(file, markdownExtensions);
          if (!markdown && !codeExtensions.has(extname(file).toLowerCase())) return;
          visited.add(file);
          const source = await readFile(file, "utf8");
          if (markdown) {
            found.set(file, { documentPath: file, source });
            return;
          }
          const compiled = await transformWithOxc(source, file, {
            jsx: { runtime: "automatic", importSource: "hono/jsx" },
          });
          const ast = parseAst(compiled.code);
          for (const node of ast.body) {
            if (
              !("source" in node) ||
              !node.source ||
              !("value" in node.source) ||
              typeof node.source.value !== "string"
            )
              continue;
            const target =
              (await resolveLayoutImport?.(node.source.value, file)) ??
              (node.source.value.startsWith(".")
                ? resolve(dirname(file), node.source.value)
                : undefined);
            if (target && !target.includes("?") && !target.startsWith("\0")) await visit(target);
          }
        }
        if (options.layout) await visit(resolve(config.root, options.layout));
        layoutDependencies.clear();
        for (const file of visited) layoutDependencies.add(file);
        return [...found.values()].map((document) => ({
          ...document,
          dependencies: [...visited],
        }));
      },
    }),
    ...(options.routes === false
      ? []
      : [
          createOxContentCustomHostPlugin({
            host,
            oxContent,
            ssrStylesheets: options.layout ? { modules: [options.layout] } : false,
            build: { transformHtml: false },
            dev: {
              routeDependencies: [
                { path: oxContent.srcDir ?? "content", kind: "directory" },
                ...(options.layout ? [{ path: options.layout, kind: "file" as const }] : []),
              ],
            },
          }),
        ]),
  ];
}
