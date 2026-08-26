import type { Element, ElementContent, Root } from "hast";
import rehypeParsePlugin from "rehype-parse";
import rehypeStringifyPlugin from "rehype-stringify";
import { unified } from "unified";
import { interopDefault } from "../interop";

const rehypeParse = interopDefault(rehypeParsePlugin);
const rehypeStringify = interopDefault(rehypeStringifyPlugin);

const BLOCK_EMBED_CLASSES = new Set([
  "ox-bluesky",
  "ox-github-card",
  "ox-github-code",
  "ox-ogp-card",
  "ox-ogp-simple",
  "ox-provider-card",
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

const BLOCK_EMBED_TAGS = new Set([
  "bluesky",
  "googlemaps",
  "qiita",
  "zenn",
  "discord",
  "fediverse",
  "mastodon",
  "misskey",
  "mixi2",
  "facebook",
  "threads",
  "instagram",
  "github",
  "ogcard",
  "spotify",
  "applemusic",
  "speakerdeck",
  "audio",
  "video",
  "stackblitz",
  "tweet",
  "webcontainer",
  "xpost",
  "youtube",
]);

export async function normalizeBlockEmbedParagraphs(html: string): Promise<string> {
  if (!shouldNormalizeBlockEmbeds(html)) return html;

  const result = await unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeBlockEmbedParagraphs)
    .use(rehypeStringify)
    .process(html);

  return String(result);
}

export function isBlockEmbedElement(node: ElementContent): node is Element {
  if (node.type !== "element") return false;
  return (
    BLOCK_EMBED_TAGS.has(node.tagName.toLowerCase()) ||
    classList(node).some((name) => BLOCK_EMBED_CLASSES.has(name))
  );
}

function shouldNormalizeBlockEmbeds(html: string): boolean {
  return (
    /<p[\s>]/i.test(html) &&
    (/\box-(?:apple-music|audio|bluesky|github|ogp|provider-card|speaker-deck|spotify|stackblitz|tweet|video|webcontainer|youtube)\b/i.test(
      html,
    ) ||
      /<(?:applemusic|audio|bluesky|discord|facebook|fediverse|github|googlemaps|instagram|mastodon|misskey|mixi2|ogcard|qiita|speakerdeck|spotify|stackblitz|threads|tweet|video|webcontainer|xpost|youtube|zenn)[\s/>]/i.test(
        html,
      ))
  );
}

function rehypeBlockEmbedParagraphs() {
  return (tree: Root) => {
    rewriteChildren(tree);
  };
}

function rewriteChildren(parent: Root | Element): void {
  const next: ElementContent[] = [];

  for (const child of parent.children) {
    if (child.type === "element" && child.children.length > 0) {
      rewriteChildren(child);
    }

    if (isParagraph(child)) {
      next.push(...splitParagraph(child));
    } else {
      next.push(child);
    }
  }

  parent.children = next;
}

function splitParagraph(paragraph: Element): ElementContent[] {
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
    if (isBlockEmbedElement(child)) {
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
