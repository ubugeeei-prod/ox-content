import { readFile, writeFile } from "node:fs/promises";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Plugin, ViteDevServer } from "vite";
import { resolveLanguage } from "./catalog";
import {
  resolveCodePlayOptions,
  type RawCodePlayOptions,
  type ResolvedCodePlayOptions,
} from "./config";
import { enhanceGeneratedModule, enhancePlayHtml } from "./html";
import { parseCodePlayTags, parsePlayFences, rewritePlayFences } from "./markdown";
import { decodePayload, encodePayload } from "./payload";
import { payloadFromFence } from "./payload-factory";
import { proxy, typecheckProxy } from "./plugin-proxy";

export type CodePlayPluginOptions = RawCodePlayOptions;

const VIRTUAL_ID = "virtual:ox-content/code-play";
const RESOLVED_VIRTUAL = `\0${VIRTUAL_ID}`;
const MARKDOWN_RE = /\.(?:md|markdown|mdx)(?:$|\?)/i;

export function codePlay(options: CodePlayPluginOptions = {}): Plugin {
  const resolved = resolveCodePlayOptions({
    ...options,
    endpoints: {
      ...options.endpoints,
      typecheck: options.endpoints?.typecheck ?? "/__ox-code-play/typecheck",
    },
  });
  let base = resolved.base;
  let command: "build" | "serve" = "serve";
  let outDir = resolved.outDir;
  let root = process.cwd();

  const enhanceOptions = (
    matchFences?: Array<{ language: string; code: string; payload: string }>,
  ) => ({
    scriptSrc: `${base}ox-code-play.js`,
    decodePayload,
    encodePayload,
    matchFences,
  });

  return {
    name: "@ox-content/code-play",

    configResolved(config) {
      command = config.command;
      root = config.root;
      base = resolved.base === "/" ? normalizeBase(config.base) : resolved.base;
      outDir = resolved.outDir ?? config.build.outDir;
    },

    resolveId(id) {
      return id === VIRTUAL_ID ? RESOLVED_VIRTUAL : null;
    },

    load(id) {
      if (id !== RESOLVED_VIRTUAL) {
        return null;
      }
      return `export { hydrateCodePlay, mountCodePlay } from ${JSON.stringify(resolveHydrateSpecifier())};`;
    },

    transform(code, id) {
      if (!MARKDOWN_RE.test(id)) {
        if (code.includes("export const html = ") && code.includes("ox-code-play:")) {
          return enhanceGeneratedModule(code, enhanceOptions());
        }
        return null;
      }
      const rewritten = rewritePlayFences(code, (fence) => {
        const definition = resolveLanguage(fence.language);
        if (!definition || !resolved.languages.has(definition.id)) {
          return null;
        }
        return encodePayload(payloadFromFence(fence, resolved));
      });
      return rewritten === code ? null : rewritten;
    },

    configureServer(server) {
      if (resolved.proxy) {
        mountProxies(server, resolved.endpoints.rust, resolved.endpoints.go);
      }
      server.middlewares.use(async (req, res, next) => {
        const urlPath = req.url?.split("?")[0] ?? "";
        if (urlPath === `${base}ox-code-play.js`.replace(/\/{2,}/g, "/")) {
          const file = resolveClientFile();
          if (!file) {
            return next();
          }
          res.setHeader("Content-Type", "text/javascript; charset=utf-8");
          res.end(await readFile(file, "utf8"));
          return;
        }
        interceptHtml(res, (html) =>
          enhanceHtmlForUrl(urlPath, html, root, resolved, enhanceOptions),
        );
        next();
      });
    },

    async generateBundle() {
      const file = resolveClientFile();
      if (!file || command !== "build") {
        return;
      }
      this.emitFile({
        type: "asset",
        fileName: "ox-code-play.js",
        source: await readFile(file, "utf8"),
      });
    },

    async closeBundle() {
      if (command !== "build") {
        return;
      }
      const srcDir = path.resolve(root, resolved.srcDir ?? "docs");
      const destination = path.resolve(root, outDir ?? "dist");
      if (!existsSync(srcDir) || !existsSync(destination)) {
        return;
      }
      await enhanceWrittenPages(srcDir, destination, resolved, enhanceOptions);
    },
  };
}

function interceptHtml(
  res: import("node:http").ServerResponse,
  enhance: (html: string) => string | undefined,
): void {
  const originalEnd = res.end.bind(res);
  res.end = ((chunk?: unknown, encoding?: BufferEncoding | (() => void), cb?: () => void) => {
    const html =
      typeof chunk === "string"
        ? chunk
        : Buffer.isBuffer(chunk)
          ? chunk.toString("utf8")
          : undefined;
    const type = String(res.getHeader("content-type") ?? "");
    if (html && (type.includes("html") || html.includes("<pre"))) {
      const next = enhance(html);
      if (next && next !== html) {
        if (typeof encoding === "function") {
          return originalEnd(next, encoding);
        }
        return originalEnd(next, encoding ?? "utf8", cb);
      }
    }
    if (typeof encoding === "function") {
      return originalEnd(chunk as never, encoding);
    }
    return originalEnd(chunk as never, encoding ?? "utf8", cb);
  }) as typeof res.end;
}

function enhanceHtmlForUrl(
  urlPath: string,
  html: string,
  root: string,
  resolved: ResolvedCodePlayOptions,
  enhance: (matchFences?: Array<{ language: string; code: string; payload: string }>) => {
    scriptSrc: string;
    decodePayload: typeof decodePayload;
    encodePayload: typeof encodePayload;
    matchFences?: Array<{ language: string; code: string; payload: string }>;
  },
): string | undefined {
  const markdownPath = urlToMarkdown(urlPath, root, resolved.srcDir ?? "docs", resolved.base);
  if (!markdownPath || !existsSync(markdownPath)) {
    return undefined;
  }
  const source = readFileSync(markdownPath, "utf8");
  const fences = [...parsePlayFences(source), ...parseCodePlayTags(source)].filter((fence) => {
    const definition = resolveLanguage(fence.language);
    return Boolean(definition && resolved.languages.has(definition.id));
  });
  if (fences.length === 0) {
    return undefined;
  }
  return enhancePlayHtml(
    html,
    enhance(
      fences.map((fence) => ({
        language: fence.language,
        code: fence.code,
        payload: encodePayload(payloadFromFence(fence, resolved)),
      })),
    ),
  );
}

function urlToMarkdown(
  urlPath: string,
  root: string,
  srcDir: string,
  base: string,
): string | undefined {
  let relative = urlPath;
  if (base !== "/" && relative.startsWith(base)) {
    relative = relative.slice(base.length);
  }
  relative = relative.replace(/^\//, "").replace(/\.html$/, "");
  if (!relative || relative.includes("..")) {
    return undefined;
  }
  const candidates = [
    path.resolve(root, srcDir, `${relative}.md`),
    path.resolve(root, srcDir, relative, "index.md"),
  ];
  return candidates.find((candidate) => existsSync(candidate));
}

function mountProxies(server: ViteDevServer, rustUrl: string, goUrl: string): void {
  server.middlewares.use("/__ox-code-play/rust", (req, res) => {
    void proxy(req, res, rustUrl, "application/json");
  });
  server.middlewares.use("/__ox-code-play/go", (req, res) => {
    void proxy(req, res, goUrl, "application/x-www-form-urlencoded");
  });
  server.middlewares.use("/__ox-code-play/typecheck", (req, res) => {
    void typecheckProxy(req, res);
  });
}

async function enhanceWrittenPages(
  srcDir: string,
  outDir: string,
  resolved: ResolvedCodePlayOptions,
  enhance: (matchFences?: Array<{ language: string; code: string; payload: string }>) => {
    scriptSrc: string;
    decodePayload: typeof decodePayload;
    encodePayload: typeof encodePayload;
    matchFences?: Array<{ language: string; code: string; payload: string }>;
  },
): Promise<void> {
  for (const file of walkFiles(srcDir)) {
    if (!MARKDOWN_RE.test(file)) {
      continue;
    }
    const source = await readFile(file, "utf8");
    const fences = [...parsePlayFences(source), ...parseCodePlayTags(source)].filter((fence) => {
      const definition = resolveLanguage(fence.language);
      return Boolean(definition && resolved.languages.has(definition.id));
    });
    if (fences.length === 0) {
      continue;
    }
    const htmlPath = guessHtmlPath(file, srcDir, outDir);
    if (!htmlPath || !existsSync(htmlPath)) {
      continue;
    }
    const html = await readFile(htmlPath, "utf8");
    const enhanced = enhancePlayHtml(
      html,
      enhance(
        fences.map((fence) => ({
          language: fence.language,
          code: fence.code,
          payload: encodePayload(payloadFromFence(fence, resolved)),
        })),
      ),
    );
    if (enhanced !== html) {
      await writeFile(htmlPath, enhanced);
    }
  }
}

function guessHtmlPath(file: string, srcDir: string, outDir: string): string | undefined {
  const relative = path.relative(srcDir, file).replace(/\.(?:md|markdown|mdx)$/i, "");
  const candidates = [
    path.join(outDir, `${relative}.html`),
    path.join(outDir, relative, "index.html"),
  ];
  return candidates.find((candidate) => existsSync(candidate));
}

function walkFiles(dir: string): string[] {
  const entries = readdirSync(dir);
  const files: string[] = [];
  for (const entry of entries) {
    const full = path.join(dir, entry);
    const stat = statSync(full);
    if (stat.isDirectory()) {
      files.push(...walkFiles(full));
    } else {
      files.push(full);
    }
  }
  return files;
}

function resolveClientFile(): string | undefined {
  const candidates = [
    fileURLToPath(new URL("./hydrate.mjs", import.meta.url)),
    fileURLToPath(new URL("./hydrate.js", import.meta.url)),
  ];
  return candidates.find((candidate) => existsSync(candidate));
}

function resolveHydrateSpecifier(): string {
  return resolveClientFile() ?? fileURLToPath(new URL("./hydrate.ts", import.meta.url));
}

function normalizeBase(base: string): string {
  if (!base || base === "/") {
    return "/";
  }
  return base.endsWith("/") ? base : `${base}/`;
}

export type { CodePlayPluginOptions as CodePlayOptions };
