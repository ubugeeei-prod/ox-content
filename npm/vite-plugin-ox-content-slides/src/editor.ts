import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { IncomingMessage, ServerResponse } from "node:http";
import type { ViteDevServer } from "vite";
import { buildSlideDecks } from "./decks";
import { renderSlideEditorHtml } from "./editor-html";
import type { ResolvedSlidesPluginOptions } from "./internal-types";
import { normalizeExtension } from "./path-utils";

interface EditorSlide {
  filePath: string;
  title: string;
  slideNumber: number;
  href: string;
  presenterHref?: string;
}

interface EditorDeck {
  slug: string;
  title: string;
  slides: EditorSlide[];
}

interface EditorPayload {
  filePath: string;
  source: string;
}

interface NewSlidePayload {
  after?: string;
}

interface SlideEditorServerOptions {
  server: ViteDevServer;
  options: ResolvedSlidesPluginOptions;
  root: string;
  invalidateArtifacts: () => void;
}

function slidesDir(root: string, options: ResolvedSlidesPluginOptions): string {
  return path.resolve(root, options.srcDir);
}

function toPosixPath(value: string): string {
  return value.split(path.sep).join("/");
}

function assertInside(parent: string, child: string): void {
  const relative = path.relative(parent, child);
  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    throw new Error("Slide source must stay inside srcDir.");
  }
}

function sourceId(root: string, options: ResolvedSlidesPluginOptions, sourcePath: string): string {
  return toPosixPath(path.relative(slidesDir(root, options), sourcePath));
}

function resolveSourcePath(
  root: string,
  options: ResolvedSlidesPluginOptions,
  filePath: string,
): string {
  const dir = slidesDir(root, options);
  const resolved = path.resolve(dir, filePath);
  assertInside(dir, resolved);
  return resolved;
}

function requestPath(req: IncomingMessage, options: ResolvedSlidesPluginOptions): string {
  const pathname = new URL(req.url ?? "/", "http://ox-content.local").pathname;
  const withoutBase =
    options.baseHref !== "/" && pathname.startsWith(options.baseHref)
      ? `/${pathname.slice(options.baseHref.length)}`
      : pathname;
  return withoutBase.endsWith("/") ? withoutBase.slice(0, -1) || "/" : withoutBase;
}

function sendJson(res: ServerResponse, payload: unknown, statusCode = 200): void {
  res.statusCode = statusCode;
  res.setHeader("Content-Type", "application/json");
  res.end(JSON.stringify(payload));
}

function sendError(res: ServerResponse, error: unknown): void {
  res.statusCode = 400;
  res.setHeader("Content-Type", "text/plain");
  res.end(error instanceof Error ? error.message : String(error));
}

async function readJson<T>(req: IncomingMessage): Promise<T> {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }
  return JSON.parse(Buffer.concat(chunks).toString("utf-8")) as T;
}

async function editorDecks(
  root: string,
  options: ResolvedSlidesPluginOptions,
): Promise<EditorDeck[]> {
  const decks = await buildSlideDecks(options, root);
  return decks.map((deck) => ({
    slug: deck.slug,
    title: deck.title,
    slides: deck.slides.map((slide) => ({
      filePath: sourceId(root, options, slide.sourcePath),
      title: slide.title,
      slideNumber: slide.slideNumber,
      href: slide.href,
      presenterHref: slide.presenterHref,
    })),
  }));
}

function findEditorSlide(decks: EditorDeck[], filePath: string): EditorSlide | undefined {
  return decks.flatMap((deck) => deck.slides).find((slide) => slide.filePath === filePath);
}

async function readSource(root: string, options: ResolvedSlidesPluginOptions, filePath: string) {
  const sourcePath = resolveSourcePath(root, options, filePath);
  const decks = await editorDecks(root, options);
  const slide = findEditorSlide(decks, filePath);
  if (!slide) {
    throw new Error(`Slide source is not part of the current deck graph: ${filePath}`);
  }

  return {
    slide,
    source: await fs.readFile(sourcePath, "utf-8"),
  };
}

async function writeSource(
  root: string,
  options: ResolvedSlidesPluginOptions,
  payload: EditorPayload,
): Promise<{ filePath: string; sourcePath: string }> {
  const sourcePath = resolveSourcePath(root, options, payload.filePath);
  await fs.writeFile(sourcePath, payload.source, "utf-8");
  return { filePath: payload.filePath, sourcePath };
}

async function createSlide(
  root: string,
  options: ResolvedSlidesPluginOptions,
  payload: NewSlidePayload,
): Promise<{ filePath: string; sourcePath: string }> {
  const dir = payload.after
    ? path.dirname(resolveSourcePath(root, options, payload.after))
    : slidesDir(root, options);
  assertInside(slidesDir(root, options), dir);
  await fs.mkdir(dir, { recursive: true });

  const entries = await fs.readdir(dir, { withFileTypes: true });
  const highest = entries.reduce((max, entry) => {
    const ext = normalizeExtension(path.extname(entry.name));
    const stem = path.basename(entry.name, ext);
    return entry.isFile() && ext === ".md" && /^\d+$/.test(stem)
      ? Math.max(max, Number(stem))
      : max;
  }, 0);
  const slideNumber = highest + 1;
  const fileName = `${String(slideNumber).padStart(4, "0")}.md`;
  const sourcePath = path.join(dir, fileName);
  const title = `Slide ${slideNumber}`;
  const source = `---\ntitle: "${title}"\nlayout: "stack"\nalign: "start"\ndensity: "balanced"\n---\n\n# ${title}\n\n- Write the next point here.\n\n<!-- Notes:\nAdd speaker notes here.\n-->\n`;
  await fs.writeFile(sourcePath, source, { encoding: "utf-8", flag: "wx" });

  return { filePath: sourceId(root, options, sourcePath), sourcePath };
}

function isEditorPage(route: string, prefix: string): boolean {
  return route === prefix || route === `${prefix}/index.html`;
}

/**
 * Mounts the dev-only GUI editor and its source-file API.
 */
export function configureSlideEditor({
  server,
  options,
  root,
  invalidateArtifacts,
}: SlideEditorServerOptions): void {
  if (!options.editor.enabled) {
    return;
  }

  server.middlewares.use(async (req, res, next) => {
    const route = requestPath(req, options);
    const method = req.method ?? "GET";

    if (method === "GET" && isEditorPage(route, options.editor.routePrefix)) {
      res.setHeader("Content-Type", "text/html");
      res.end(renderSlideEditorHtml(options.editor.apiPrefix));
      return;
    }

    if (!route.startsWith(options.editor.apiPrefix)) {
      next();
      return;
    }

    try {
      if (method === "GET" && route === `${options.editor.apiPrefix}/decks`) {
        sendJson(res, { decks: await editorDecks(root, options) });
        return;
      }

      if (method === "GET" && route === `${options.editor.apiPrefix}/source`) {
        const file = new URL(req.url ?? "/", "http://ox-content.local").searchParams.get("file");
        if (!file) throw new Error("Missing file query parameter.");
        sendJson(res, await readSource(root, options, file));
        return;
      }

      if (method === "PUT" && route === `${options.editor.apiPrefix}/source`) {
        const result = await writeSource(root, options, await readJson<EditorPayload>(req));
        invalidateArtifacts();
        server.ws.send({
          type: "custom",
          event: "ox-content:slides:update",
          data: { file: result.sourcePath },
        });
        sendJson(res, { filePath: result.filePath });
        return;
      }

      if (method === "POST" && route === `${options.editor.apiPrefix}/slides`) {
        const result = await createSlide(root, options, await readJson<NewSlidePayload>(req));
        invalidateArtifacts();
        server.watcher.add(result.sourcePath);
        server.ws.send({
          type: "custom",
          event: "ox-content:slides:update",
          data: { file: result.sourcePath },
        });
        sendJson(res, { filePath: result.filePath }, 201);
        return;
      }

      next();
    } catch (error) {
      sendError(res, error);
    }
  });
}
