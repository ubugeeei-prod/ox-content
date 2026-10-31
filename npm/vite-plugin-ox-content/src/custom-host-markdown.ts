import * as path from "node:path";
import { hasIslands, resetIslandCounter, transformIslands } from "./island";
import { resetTabGroupCounter, transformAllPlugins, type TransformAllOptions } from "./plugins";
import { protectMermaidSvgs, restoreMermaidSvgs } from "./plugins/mermaid-protect";
import {
  applyReaderChromeHtml,
  type ReaderChromeInput,
  resolveReaderChromeInput,
} from "./reader-chrome";
import { normalizeRoutePrefix } from "./ssg";
import { transformMarkdown } from "./transform";
import type { TransformResult } from "./types";
import type {
  MaybePromise,
  OxContentCustomHostAssetsContext,
  OxContentCustomHostBaseContext,
  OxContentCustomHostDependency,
} from "./custom-host-types";
import { normalizeBase } from "./custom-host-utils";

export interface OxContentCustomHostMarkdownRenderer {
  render<Metadata = unknown>(
    input: OxContentCustomHostMarkdownInput<Metadata>,
  ): Promise<OxContentCustomHostMarkdownResult<Metadata>>;
}

export interface OxContentCustomHostMarkdownInput<Metadata = unknown> {
  source: string;
  documentPath: string;
  convertMdLinks?: boolean;
  base?: string;
  readerChrome?: ReaderChromeInput;
  dependencies?: readonly OxContentCustomHostDependency[];
  renderHtml?: (
    context: OxContentCustomHostMarkdownRenderContext,
  ) => MaybePromise<string | OxContentCustomHostMarkdownRenderResult<Metadata> | undefined | void>;
}

export interface OxContentCustomHostMarkdownRenderContext {
  html: string;
  transform: TransformResult;
  source: string;
  documentPath: string;
  root: string;
  srcDir: string;
  contentRoot: string;
  base: string;
  mode: OxContentCustomHostBaseContext["mode"];
  loadModule(moduleId: string): Promise<unknown>;
  assets: OxContentCustomHostAssetsContext;
}

export interface OxContentCustomHostMarkdownRenderResult<Metadata = unknown> {
  html: string;
  metadata?: Metadata;
  dependencies?: readonly OxContentCustomHostDependency[];
}

export interface OxContentCustomHostMarkdownResult<Metadata = unknown> {
  html: string;
  markdownHtml: string;
  transform: TransformResult;
  frontmatter: TransformResult["frontmatter"];
  toc: TransformResult["toc"];
  imports: TransformResult["imports"];
  exports: TransformResult["exports"];
  components: TransformResult["components"];
  metadata?: Metadata;
  dependencies: OxContentCustomHostDependency[];
}

export function createCustomHostMarkdownRenderer(
  context: Omit<OxContentCustomHostBaseContext, "markdown">,
): OxContentCustomHostMarkdownRenderer {
  return {
    async render(input) {
      const documentPath = path.resolve(context.root, input.documentPath);
      const srcDir = path.resolve(context.root, context.options.srcDir);
      const base = input.base ?? publicBase(context.base, context.options.ssg.routePrefix);

      resetTabAndIslandCounters();
      const transform = await transformMarkdown(input.source, documentPath, context.options, {
        deferModuleCode: true,
        convertMdLinks: input.convertMdLinks ?? true,
        baseUrl: base,
        sourcePath: documentPath,
        srcDir,
      });

      let html = await transformCustomHostMarkdownHtml(transform.html, context);
      const markdownHtml = html;
      const dependencies: OxContentCustomHostDependency[] = [
        documentPath,
        ...(input.dependencies ?? []),
      ];
      let metadata: unknown;

      if (input.renderHtml) {
        const rendered = await input.renderHtml({
          html,
          transform,
          source: input.source,
          documentPath,
          root: context.root,
          srcDir,
          contentRoot: srcDir,
          base,
          mode: context.mode,
          loadModule: context.loadModule,
          assets: context.assets,
        });
        if (typeof rendered === "string") {
          html = rendered;
        } else if (rendered) {
          html = rendered.html;
          metadata = rendered.metadata;
          dependencies.push(...(rendered.dependencies ?? []));
        }
      }

      const readerChrome = resolveReaderChromeInput(
        input.readerChrome ?? context.options.ssg.readerChrome,
      );
      html = applyReaderChromeHtml(html, readerChrome);

      const result: OxContentCustomHostMarkdownResult<Metadata> = {
        html,
        markdownHtml,
        transform,
        frontmatter: transform.frontmatter,
        toc: transform.toc,
        imports: transform.imports,
        exports: transform.exports,
        components: transform.components,
        dependencies,
      };
      if (metadata !== undefined) {
        result.metadata = metadata as Metadata;
      }
      return result;
    },
  };
}

async function transformCustomHostMarkdownHtml(
  html: string,
  context: OxContentCustomHostBaseContext,
): Promise<string> {
  const { html: protectedHtml, svgs: mermaidSvgs } = protectMermaidSvgs(html);
  let transformedHtml = await transformAllPlugins(protectedHtml, markdownPluginOptions(context));
  if (hasIslands(transformedHtml)) {
    const islandResult = await transformIslands(transformedHtml);
    transformedHtml = islandResult.html;
  }
  return restoreMermaidSvgs(transformedHtml, mermaidSvgs);
}

function markdownPluginOptions(context: OxContentCustomHostBaseContext): TransformAllOptions {
  const embeds = context.options.embeds;
  return {
    tabs: true,
    youtube: true,
    github: embeds.github,
    openGraph: withRootCacheDir(embeds.openGraph, context.root),
    pm: embeds.pm,
    spotify: embeds.spotify,
    appleMusic: embeds.appleMusic,
    speakerDeck: embeds.speakerDeck,
    audio: embeds.audio,
    video: embeds.video,
    stackBlitz: embeds.stackBlitz,
    twitter: withRootPathOptions(embeds.twitter, context.root, ["cacheDir", "mediaOutputDir"]),
    reddit: embeds.reddit,
    bluesky: embeds.bluesky,
    googleMaps: embeds.googleMaps,
    qiita: withRootCacheDir(embeds.qiita, context.root),
    zenn: withRootCacheDir(embeds.zenn, context.root),
    packageRegistry: withRootCacheDir(embeds.packageRegistry, context.root),
    playgrounds: withRootCacheDir(embeds.playgrounds, context.root),
    vimeo: withRootCacheDir(embeds.vimeo, context.root),
    twitch: withRootCacheDir(embeds.twitch, context.root),
    discord: embeds.discord,
    fediverse: embeds.fediverse,
    facebook: embeds.facebook,
    threads: embeds.threads,
    instagram: embeds.instagram,
    webContainer: embeds.webContainer,
    loom: embeds.loom,
    asciinema: embeds.asciinema,
    figma: embeds.figma,
    note: embeds.note,
    googleSlides: embeds.googleSlides,
    mermaid: true,
    githubToken: process.env.GITHUB_TOKEN,
  };
}

function withRootCacheDir<T>(input: T, root: string): T {
  return withRootPathOptions(input, root, ["cacheDir"]);
}

function withRootPathOptions<T>(input: T, root: string, keys: readonly string[]): T {
  if (!input || typeof input !== "object") {
    return input;
  }
  let output: Record<string, unknown> | undefined;
  const record = input as Record<string, unknown>;
  for (const key of keys) {
    const value = record[key];
    if (typeof value === "string" && value && !path.isAbsolute(value)) {
      output ??= { ...record };
      output[key] = path.resolve(root, value);
    }
  }
  return (output ?? input) as T;
}

function publicBase(base: string, routePrefix?: string): string {
  const root = normalizeBase(base);
  const prefix = normalizeRoutePrefix(routePrefix);
  return prefix ? `${root}${prefix}/` : root;
}

function resetTabAndIslandCounters(): void {
  resetTabGroupCounter();
  resetIslandCounter();
}
