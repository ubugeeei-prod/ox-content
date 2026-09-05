import { headHtml, renderDocumentAssetTag } from "./document-assets-tags";
import type {
  DocumentAssetDescriptor,
  DocumentAssetManifest,
  DocumentLinkDescriptor,
  DocumentLinkInput,
  DocumentScriptDescriptor,
  DocumentScriptInput,
  DocumentStyleDescriptor,
  DocumentStylesheetInput,
  RenderDocumentAssetsInput,
  RenderDocumentAssetsResult,
} from "./document-assets";

export function renderDocumentAssets(input: RenderDocumentAssetsInput): RenderDocumentAssetsResult {
  const context = createRenderContext(input);
  const tags: DocumentAssetDescriptor[] = [];

  for (const link of input.links ?? []) {
    tags.push(linkDescriptor(link, context));
  }
  for (const preload of input.selfHostedAssets?.preloads ?? []) {
    tags.push({
      kind: "link",
      rel: "preload",
      href: withBase(context.base, preload.href),
      as: preload.as,
      type: preload.type,
      crossorigin: preload.crossorigin,
    });
  }
  for (const href of input.selfHostedAssets?.stylesheets ?? []) {
    tags.push(stylesheetDescriptor(href, context));
  }
  for (const style of input.sharedStyles ?? []) {
    tags.push(stylesheetDescriptor(style, context));
  }
  for (const style of input.pageStyles ?? []) {
    tags.push(stylesheetDescriptor(style, context));
  }
  for (const style of input.islandStyles ?? []) {
    tags.push(stylesheetDescriptor(style, context));
  }
  for (const style of input.inlineStyles ?? []) {
    tags.push(stylesheetDescriptor(style, context));
  }
  for (const entry of input.clientEntries ?? []) {
    tags.push(...clientEntryDescriptors(entry, input.manifest, context));
  }
  for (const script of input.scripts ?? []) {
    tags.push(scriptDescriptor(script, context));
  }

  const deduped = dedupeTags(tags);
  const links = deduped.filter((tag): tag is DocumentLinkDescriptor => tag.kind === "link");
  const styles = deduped.filter((tag): tag is DocumentStyleDescriptor => tag.kind === "style");
  const scripts = deduped.filter((tag): tag is DocumentScriptDescriptor => tag.kind === "script");
  const parts = [headHtml(input.head), ...deduped.map(renderDocumentAssetTag)].filter(Boolean);

  return {
    links,
    styles,
    scripts,
    tags: deduped,
    headHtml: parts.join("\n"),
  };
}

function createRenderContext(input: RenderDocumentAssetsInput) {
  const nonce =
    typeof input.nonce === "string"
      ? { style: input.nonce, script: input.nonce }
      : (input.nonce ?? {});
  return {
    base: normalizeBase(input.base),
    styleNonce: nonce.style,
    scriptNonce: nonce.script,
    crossorigin: input.crossorigin,
  };
}

function linkDescriptor(input: DocumentLinkInput, context: ReturnType<typeof createRenderContext>) {
  if (typeof input === "string") {
    return {
      kind: "link",
      rel: "stylesheet",
      href: withBase(context.base, input),
    } satisfies DocumentLinkDescriptor;
  }
  return {
    ...input,
    kind: "link",
    href: withBase(context.base, input.href),
  } satisfies DocumentLinkDescriptor;
}

function stylesheetDescriptor(
  input: DocumentStylesheetInput,
  context: ReturnType<typeof createRenderContext>,
): DocumentStyleDescriptor {
  if (typeof input === "string") {
    return { kind: "style", href: withBase(context.base, input), nonce: context.styleNonce };
  }
  return {
    ...input,
    kind: "style",
    href: input.href ? withBase(context.base, input.href) : undefined,
    nonce: input.nonce ?? (input.href ? undefined : context.styleNonce),
  };
}

function scriptDescriptor(
  input: DocumentScriptInput,
  context: ReturnType<typeof createRenderContext>,
): DocumentScriptDescriptor {
  if (typeof input === "string") {
    return {
      kind: "script",
      src: withBase(context.base, input),
      type: "module",
      nonce: context.scriptNonce,
    };
  }
  return {
    ...input,
    kind: "script",
    src: input.src ? withBase(context.base, input.src) : undefined,
    nonce: input.nonce ?? (input.src ? undefined : context.scriptNonce),
  };
}

function clientEntryDescriptors(
  input: DocumentScriptInput,
  manifest: DocumentAssetManifest | undefined,
  context: ReturnType<typeof createRenderContext>,
): DocumentAssetDescriptor[] {
  const source = typeof input === "string" ? input : input.src;
  if (!source || !manifest) {
    return [scriptDescriptor(input, context)];
  }

  const key = manifestKey(manifest, source);
  if (!key) {
    return [scriptDescriptor(input, context)];
  }

  const descriptors: DocumentAssetDescriptor[] = [];
  const seenChunks = new Set<string>();
  const seenCss = new Set<string>();
  const visit = (chunkKey: string) => {
    if (seenChunks.has(chunkKey)) {
      return;
    }
    seenChunks.add(chunkKey);
    const chunk = manifest[chunkKey];
    if (!chunk) {
      return;
    }
    for (const imported of chunk.imports ?? []) {
      visit(imported);
    }
    for (const css of chunk.css ?? []) {
      const href = withBase(context.base, css);
      if (!seenCss.has(href)) {
        seenCss.add(href);
        descriptors.push({ kind: "style", href });
      }
    }
  };
  visit(key);

  const chunk = manifest[key];
  if (!chunk?.file) {
    return [...descriptors, scriptDescriptor(input, context)];
  }
  descriptors.push({
    ...scriptDescriptor(
      typeof input === "string" ? { src: chunk.file } : { ...input, src: chunk.file },
      context,
    ),
    type: typeof input === "string" ? "module" : (input.type ?? "module"),
    crossorigin:
      typeof input === "string" ? context.crossorigin : (input.crossorigin ?? context.crossorigin),
  });
  return descriptors;
}

function dedupeTags(tags: DocumentAssetDescriptor[]): DocumentAssetDescriptor[] {
  const seen = new Set<string>();
  const result: DocumentAssetDescriptor[] = [];
  for (const tag of tags) {
    const key = tagIdentity(tag);
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    result.push(tag);
  }
  return result;
}

function tagIdentity(tag: DocumentAssetDescriptor): string {
  if (tag.key) {
    return `${tag.kind}:key:${tag.key}`;
  }
  if (tag.kind === "link") {
    return `${tag.kind}:${tag.rel}:${tag.href}`;
  }
  if (tag.kind === "style") {
    return tag.href ? `style:${tag.href}` : `style:inline:${tag.content ?? ""}`;
  }
  return tag.src
    ? `script:${tag.type ?? "module"}:${tag.src}`
    : `script:inline:${tag.content ?? ""}`;
}

function manifestKey(manifest: DocumentAssetManifest, moduleId: string): string | undefined {
  if (manifest[moduleId]) {
    return moduleId;
  }
  return Object.entries(manifest).find(
    ([key, chunk]) => key === moduleId || chunk.src === moduleId || chunk.file === moduleId,
  )?.[0];
}

function normalizeBase(base: string | undefined): string {
  if (!base || base === "/") {
    return "/";
  }
  const withLeading = base.startsWith("/") ? base : `/${base}`;
  return withLeading.endsWith("/") ? withLeading : `${withLeading}/`;
}

function withBase(base: string, href: string): string {
  if (isExternalOrSpecialUrl(href)) {
    return href;
  }
  if (base === "/") {
    return href.startsWith("/") ? href : `/${href}`;
  }
  if (href === base.slice(0, -1) || href.startsWith(base)) {
    return href;
  }
  return `${base}${href.replace(/^\/+/u, "")}`;
}

function isExternalOrSpecialUrl(href: string): boolean {
  return href.startsWith("#") || href.startsWith("//") || /^[a-z][a-z0-9+.-]*:/iu.test(href);
}
