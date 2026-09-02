import type { Element, ElementContent, Root } from "hast";
import rehypeParsePlugin from "rehype-parse";
import rehypeStringifyPlugin from "rehype-stringify";
import { unified } from "unified";
import { interopDefault } from "../interop";
import { importNapiModule } from "../napi";

const rehypeParse = interopDefault(rehypeParsePlugin);
const rehypeStringify = interopDefault(rehypeStringifyPlugin);

const BLOCK_EMBED_CLASSES = new Set([
  "ox-bluesky",
  "ox-github-card",
  "ox-github-code",
  "ox-ogp-card",
  "ox-ogp-simple",
  "ox-provider-card",
  "ox-reddit-card",
  "ox-spotify",
  "ox-apple-music",
  "ox-speaker-deck",
  "ox-audio",
  "ox-video",
  "ox-stackblitz",
  "ox-tweet",
  "ox-webcontainer",
  "ox-youtube",
]);

interface NapiMediaModule {
  mediaEmbedTags(): { name: string; pascalOnly: boolean }[];
}

/**
 * Block embeds this package owns, which the Rust provider registry cannot name.
 *
 * `NotByAI` is deliberately absent: the badge is an inline mark on the sentence
 * that carries it, so lifting it out of its paragraph would break the line it
 * belongs to.
 */
const TYPESCRIPT_ONLY_BLOCK_EMBED_TAGS = ["github", "ogcard", "reddit", "youtube"] as const;

let cachedTags: ReadonlySet<string> | undefined;
let cachedTagPattern: RegExp | undefined;
let cachedClassPattern: RegExp | undefined;

/**
 * Every tag whose embed is a block, taken from the registry rather than copied.
 *
 * A hand-kept copy drifts, and the drift is not cosmetic: the seven providers
 * added after this list was written -- `note` among them -- stayed inside their
 * paragraph, and a card whose outer element is an `<a>` wrapping `<div>`s does
 * not survive that. The parser closes the paragraph at the first `<div>` and
 * reopens the anchor around each block it held, so one card came out as an
 * empty link followed by three underlined ones with no card chrome at all.
 */
async function blockEmbedTags(): Promise<ReadonlySet<string>> {
  if (cachedTags) return cachedTags;
  const mod = (await importNapiModule()) as unknown as NapiMediaModule;
  cachedTags = new Set([
    ...mod.mediaEmbedTags().map((tag) => tag.name.toLowerCase()),
    ...TYPESCRIPT_ONLY_BLOCK_EMBED_TAGS,
  ]);
  return cachedTags;
}

export async function normalizeBlockEmbedParagraphs(html: string): Promise<string> {
  if (!/<p[\s>]/i.test(html)) return html;

  const tags = await blockEmbedTags();
  if (!shouldNormalizeBlockEmbeds(html, tags)) return html;

  const result = await unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeBlockEmbedParagraphs(tags))
    .use(rehypeStringify)
    .process(html);

  return String(result);
}

export function isBlockEmbedElement(
  node: ElementContent,
  tags: ReadonlySet<string>,
): node is Element {
  if (node.type !== "element") return false;
  return (
    tags.has(node.tagName.toLowerCase()) ||
    classList(node).some((name) => BLOCK_EMBED_CLASSES.has(name))
  );
}

function shouldNormalizeBlockEmbeds(html: string, tags: ReadonlySet<string>): boolean {
  return classMarkerPattern().test(html) || tagMarkerPattern(tags).test(html);
}

function classMarkerPattern(): RegExp {
  cachedClassPattern ??= new RegExp(`\\b(?:${[...BLOCK_EMBED_CLASSES].join("|")})\\b`, "i");
  return cachedClassPattern;
}

function tagMarkerPattern(tags: ReadonlySet<string>): RegExp {
  cachedTagPattern ??= new RegExp(`<(?:${[...tags].join("|")})[\\s/>]`, "i");
  return cachedTagPattern;
}

function rehypeBlockEmbedParagraphs(tags: ReadonlySet<string>) {
  return () => (tree: Root) => {
    rewriteChildren(tree, tags);
  };
}

function rewriteChildren(parent: Root | Element, tags: ReadonlySet<string>): void {
  const next: ElementContent[] = [];

  for (const child of parent.children) {
    if (child.type === "element" && child.children.length > 0) {
      rewriteChildren(child, tags);
    }

    if (isParagraph(child)) {
      next.push(...splitParagraph(child, tags));
    } else {
      next.push(child);
    }
  }

  parent.children = next;
}

function splitParagraph(paragraph: Element, tags: ReadonlySet<string>): ElementContent[] {
  const replacement: ElementContent[] = [];
  let prose: ElementContent[] = [];
  let changed = false;

  const flushProse = () => {
    if (hasMeaningfulContent(prose)) {
      replacement.push({
        ...paragraph,
        properties: { ...paragraph.properties },
        children: prose,
      });
    }
    prose = [];
  };

  for (const child of paragraph.children) {
    if (isBlockEmbedElement(child, tags)) {
      changed = true;
      flushProse();
      replacement.push(child);
      continue;
    }

    prose.push(child);
  }

  flushProse();

  if (changed) return replacement;
  return hasMeaningfulContent(paragraph.children) ? [paragraph] : [];
}

function isParagraph(node: ElementContent): node is Element {
  return node.type === "element" && node.tagName.toLowerCase() === "p";
}

function hasMeaningfulContent(children: ElementContent[]): boolean {
  return children.some((child) => child.type !== "text" || child.value.trim() !== "");
}

function classList(node: Element): string[] {
  const value = node.properties?.className;
  if (Array.isArray(value)) return value.map(String);
  if (typeof value === "string") return value.split(/\s+/).filter(Boolean);
  return [];
}
