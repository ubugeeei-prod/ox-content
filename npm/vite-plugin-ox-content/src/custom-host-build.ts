import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { ResolvedConfig, ViteDevServer } from "vite";
import { writeSelfHostedAssets } from "./assets";
import { createAssetsContext, readClientManifest, writeThemeTokens } from "./custom-host-assets";
import {
  createBaseContext,
  createHostLoaderServer,
  loadHost,
  loadRoutes,
} from "./custom-host-loader";
import { normalizeRenderResult, renderRoute } from "./custom-host-render";
import type {
  OxContentCustomHostBaseContext,
  OxContentCustomHostModule,
  OxContentCustomHostOptions,
  OxContentCustomHostRoute,
  RenderedBuildRoute,
  ResolvedThemeTokens,
} from "./custom-host-types";
import {
  claimOutput,
  contentTypeFromHeaders,
  isHtmlContentType,
  normalizeRoutePath,
  resolveInputPath,
  resolveOutDir,
  resolveOutputPath,
  resultBody,
  routeOutputPath,
  shouldTransformHtml,
  ssgUrlPath,
} from "./custom-host-utils";
import { writeRedirectOutputs } from "./redirect-outputs";
import {
  planSsgOutputs,
  writeFeedFiles,
  writeMarkdownCompanions,
  writeResourceFiles,
  writeSiteMapFiles,
} from "./ssg-output";
import type { OxContentOptions, ResolvedOptions, SsgOutputPageInput } from "./types";

export async function runCustomHostBuild(
  config: ResolvedConfig,
  input: OxContentCustomHostOptions,
  options: ResolvedOptions,
  themeTokens: ResolvedThemeTokens | undefined,
  rawOptions: OxContentOptions,
): Promise<void> {
  const root = config.root;
  const outDir = resolveOutDir(config, options, root);
  const loaderServer = await createHostLoaderServer(config);
  const clientManifest = await readClientManifest(outDir);
  const assets = createAssetsContext(options, outDir, clientManifest, themeTokens);
  const loadModule = (moduleId: string) => loaderServer.ssrLoadModule(moduleId);

  try {
    const baseContext = createBaseContext("build", root, outDir, options, loadModule, assets);
    const host = await loadHost(input.host, loadModule, root);
    const routes = await loadRoutes(host, baseContext);
    const rendered = await renderBuildRoutes(host, routes, baseContext, input, loaderServer);

    await writeThemeTokens(outDir, themeTokens);
    await writeCoordinatedOutputs(rendered, options, rawOptions, root, outDir);
  } finally {
    await loaderServer.close();
  }
}

async function renderBuildRoutes(
  host: OxContentCustomHostModule,
  routes: readonly OxContentCustomHostRoute[],
  context: OxContentCustomHostBaseContext,
  input: OxContentCustomHostOptions,
  server: ViteDevServer,
): Promise<RenderedBuildRoute[]> {
  const rendered: RenderedBuildRoute[] = [];
  const owners = new Map<string, string>();
  for (const route of routes) {
    const routePath = normalizeRoutePath(route.path);
    const request = new Request(new URL(routePath, "http://localhost"), { method: "GET" });
    const result = await renderRoute(host, route, context, request);
    if (!result) {
      continue;
    }
    const renderedResult = await normalizeRenderResult(result);
    const contentType =
      renderedResult.contentType ?? contentTypeFromHeaders(renderedResult.headers) ?? "text/html";
    const outputPath = renderedResult.outputPath
      ? resolveOutputPath(context.outDir, renderedResult.outputPath)
      : routeOutputPath(context.outDir, routePath, contentType);
    claimOutput(owners, outputPath, routePath);
    let body = resultBody(renderedResult);
    if (typeof body === "string" && shouldTransformHtml(contentType, input.build?.transformHtml)) {
      body = await server.transformIndexHtml(routePath, body);
    }
    rendered.push({ route, routePath, outputPath, contentType, body, result: renderedResult });
  }
  return rendered;
}

async function writeCoordinatedOutputs(
  routes: readonly RenderedBuildRoute[],
  options: ResolvedOptions,
  rawOptions: OxContentOptions,
  root: string,
  outDir: string,
): Promise<void> {
  const pages = routes.flatMap((entry): SsgOutputPageInput[] => {
    if (!isHtmlContentType(entry.contentType)) {
      return [];
    }
    return [
      {
        inputPath: resolveInputPath(entry.result.inputPath ?? entry.route.inputPath, root),
        urlPath: ssgUrlPath(entry.routePath),
        outputPath: entry.outputPath,
        html: typeof entry.body === "string" ? entry.body : new TextDecoder().decode(entry.body),
        source: entry.result.source ?? entry.route.source,
        title: entry.result.title ?? entry.route.title,
        description: entry.result.description ?? entry.route.description,
        loc: entry.result.loc,
        lastUpdated: entry.result.lastUpdated,
        draft: entry.result.draft ?? entry.route.draft,
        unlisted: entry.result.unlisted ?? entry.route.unlisted,
        frontmatter: entry.result.frontmatter ?? entry.route.frontmatter,
      },
    ];
  });

  const plan = planSsgOutputs({
    pages,
    outDir,
    root,
    srcDir: path.resolve(root, options.srcDir),
    options: rawOptions,
  });

  const selfHostedAssets = await writeSelfHostedAssets(plan.selfHostedAssets);
  for (const error of selfHostedAssets.errors) {
    console.warn(`[ox-content] ${error}`);
  }

  const resources = await writeResourceFiles(plan.resources);
  for (const error of resources.errors) {
    console.warn(`[ox-content] ${error}`);
  }
  const rewrittenHtml = new Map(
    resources.pages.map((page) => [path.resolve(page.outputPath), page.html]),
  );

  await Promise.all(
    routes.map(async (entry) => {
      const body = rewrittenHtml.get(path.resolve(entry.outputPath)) ?? entry.body;
      await fs.mkdir(path.dirname(entry.outputPath), { recursive: true });
      await fs.writeFile(entry.outputPath, body);
    }),
  );

  const redirects = await writeRedirectOutputs({
    outDir,
    base: options.base,
    redirects: options.redirects,
    routes: routes.map((entry) => ({
      path: entry.routePath,
      aliases: entry.result.aliases ?? entry.route.aliases,
      redirect: entry.result.redirect ?? entry.route.redirect,
    })),
    occupiedPaths: routes.map((entry) => entry.routePath),
  });

  const markdown = await writeMarkdownCompanions(plan.markdownCompanions);
  const feeds = await writeFeedFiles(plan.feeds);
  if (feeds.warning) {
    console.warn(feeds.warning);
  }
  const siteMaps = await writeSiteMapFiles(plan.siteMaps);
  if (siteMaps.warning) {
    console.warn(siteMaps.warning);
  }

  const count =
    routes.length +
    selfHostedAssets.files.length +
    resources.files.length +
    redirects.files.length +
    markdown.files.length +
    feeds.files.length +
    siteMaps.files.length;
  if (count > 0) {
    console.log(`[ox-content] Custom host generated ${count} output files`);
  }
}
