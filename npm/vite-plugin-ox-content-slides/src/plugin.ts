import * as path from "node:path";
import type { Plugin } from "vite";
import { buildOutput } from "./build";
import { buildSlideDecks } from "./decks";
import { configureSlideEditor } from "./editor";
import { injectViteHmrClient, renderRouteHtml } from "./html";
import type { SlideBuildArtifacts } from "./internal-types";
import { loadNapiModule } from "./napi";
import { resolveOptions } from "./options";
import { getSlideRouteLookupKey, normalizeExtension } from "./path-utils";
import type { OxContentSlidesOptions } from "./public-types";
import { buildSlideArtifacts } from "./routes";

/**
 * Creates the Ox Content slide plugin with MPA, presenter, and PDF support.
 */
export function oxContentSlides(rawOptions: OxContentSlidesOptions = {}): Plugin {
  const options = resolveOptions(rawOptions);
  const pageCache = new Map<string, string>();
  let artifactsPromise: Promise<SlideBuildArtifacts> | undefined;
  let root = process.cwd();

  const watchExtensions = new Set([
    ...options.extensions,
    ...Object.keys(options.renderers).map(normalizeExtension),
  ]);
  const isSlideFile = (file: string) =>
    file.startsWith(path.resolve(root, options.srcDir)) &&
    watchExtensions.has(normalizeExtension(path.extname(file)));
  const getArtifacts = () =>
    (artifactsPromise ??= buildSlideDecks(options, root).then((decks) =>
      buildSlideArtifacts(options, decks),
    ));
  const invalidateArtifacts = () => {
    artifactsPromise = undefined;
    pageCache.clear();
  };

  return {
    name: "ox-content:slides",
    configResolved(config) {
      root = config.root;
    },
    configureServer(server) {
      if (!options.ssg.enabled) return;

      server.watcher.add(path.resolve(root, options.srcDir));
      const invalidate = (file: string) => {
        if (!isSlideFile(file)) return;
        invalidateArtifacts();
        server.ws.send({ type: "custom", event: "ox-content:slides:update", data: { file } });
      };

      configureSlideEditor({ server, options, root, invalidateArtifacts });

      server.middlewares.use(async (req, res, next) => {
        const routeKey = req.url ? getSlideRouteLookupKey(options, req.url) : null;
        if (!routeKey) return next();

        const cached = pageCache.get(routeKey);
        if (cached) {
          res.setHeader("Content-Type", "text/html");
          res.end(cached);
          return;
        }

        const route = (await getArtifacts()).routes.get(routeKey);
        if (!route) return next();

        const html = injectViteHmrClient(
          await renderRouteHtml(options, route, await loadNapiModule()),
        );
        pageCache.set(routeKey, html);
        res.setHeader("Content-Type", "text/html");
        res.end(html);
      });

      server.watcher.on("add", invalidate);
      server.watcher.on("unlink", invalidate);
      server.watcher.on("change", invalidate);
    },
    handleHotUpdate({ file }) {
      if (isSlideFile(file)) invalidateArtifacts();
    },
    async closeBundle() {
      if (!options.ssg.enabled) return;

      const result = await buildOutput(options, root);
      if (result.files.length > 0) {
        console.log(`[ox-content:slides] Generated ${result.files.length} output files`);
      }
      for (const error of result.errors) {
        console.warn(`[ox-content:slides] ${error}`);
      }
    },
  };
}
