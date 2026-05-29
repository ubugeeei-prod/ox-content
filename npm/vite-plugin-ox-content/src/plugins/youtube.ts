/**
 * YouTube Plugin - Privacy-enhanced iframe embedding
 *
 * Transforms <YouTube> components into responsive iframe embeds
 * using youtube-nocookie.com for enhanced privacy.
 */

import { unified } from "unified";
import rehypeParse from "rehype-parse";
import rehypeStringify from "rehype-stringify";
import type { Root, Element, Properties } from "hast";

export interface YouTubeOptions {
  /** Use privacy-enhanced mode (youtube-nocookie.com). Default: true */
  privacyEnhanced?: boolean;
  /** Default aspect ratio. Default: "16/9" */
  aspectRatio?: string;
  /** Allow fullscreen. Default: true */
  allowFullscreen?: boolean;
  /** Lazy load iframe. Default: true */
  lazyLoad?: boolean;
}

const defaultOptions: Required<YouTubeOptions> = {
  privacyEnhanced: true,
  aspectRatio: "16/9",
  allowFullscreen: true,
  lazyLoad: true,
};

/**
 * Get element attribute value.
 */
function getAttribute(el: Element, name: string): string | undefined {
  const value = el.properties?.[name];
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return value.join(" ");
  return undefined;
}

/**
 * Extract YouTube video ID from various URL formats.
 */
export function extractVideoId(input: string): string | null {
  // Already a video ID (11 characters, alphanumeric + _ -)
  if (/^[a-zA-Z0-9_-]{11}$/.test(input)) {
    return input;
  }

  // Full URL patterns
  const patterns = [
    /(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/embed\/|youtube\.com\/v\/)([a-zA-Z0-9_-]{11})/,
    /youtube\.com\/shorts\/([a-zA-Z0-9_-]{11})/,
  ];

  for (const pattern of patterns) {
    const match = input.match(pattern);
    if (match) return match[1];
  }

  return null;
}

/**
 * Build YouTube embed URL with parameters.
 */
function buildEmbedUrl(
  videoId: string,
  options: Required<YouTubeOptions>,
  params?: Record<string, string>,
): string {
  const domain = options.privacyEnhanced ? "www.youtube-nocookie.com" : "www.youtube.com";
  const url = new URL(`https://${domain}/embed/${videoId}`);

  // Add any custom parameters
  if (params) {
    for (const [key, value] of Object.entries(params)) {
      url.searchParams.set(key, value);
    }
  }

  return url.toString();
}

/**
 * Create YouTube embed element.
 */
function createYouTubeElement(
  videoId: string,
  options: Required<YouTubeOptions>,
  title?: string,
  start?: string,
): Element {
  const params: Record<string, string> = {};
  if (start) {
    params.start = start;
  }

  const embedUrl = buildEmbedUrl(videoId, options, params);

  const iframeProps: Properties = {
    src: embedUrl,
    title: title || `YouTube video ${videoId}`,
    allow:
      "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share",
    referrerpolicy: "strict-origin-when-cross-origin",
    allowfullscreen: options.allowFullscreen || undefined,
    loading: options.lazyLoad ? "lazy" : undefined,
  };

  const iframe: Element = {
    type: "element",
    tagName: "iframe",
    properties: iframeProps,
    children: [],
  };

  return {
    type: "element",
    tagName: "div",
    properties: {
      className: ["ox-youtube"],
      style: `aspect-ratio: ${options.aspectRatio};`,
    },
    children: [iframe],
  };
}

/**
 * Rehype plugin to transform YouTube components.
 */
function rehypeYouTube(options: Required<YouTubeOptions>) {
  return (tree: Root) => {
    const visit = (node: Root | Element) => {
      if ("children" in node) {
        for (let i = 0; i < node.children.length; i++) {
          const child = node.children[i];

          if (child.type === "element") {
            // Check for <YouTube> component
            if (child.tagName.toLowerCase() === "youtube") {
              const id = getAttribute(child, "id");
              const url = getAttribute(child, "url");
              const title = getAttribute(child, "title");
              const start = getAttribute(child, "start");

              // Extract video ID from id or url attribute
              const videoId = id ? extractVideoId(id) : url ? extractVideoId(url) : null;

              if (videoId) {
                const youtubeElement = createYouTubeElement(videoId, options, title, start);
                node.children[i] = youtubeElement;
              }
            } else {
              visit(child);
            }
          }
        }
      }
    };

    visit(tree);
  };
}

/**
 * Transform YouTube components in HTML.
 */
export async function transformYouTube(html: string, options?: YouTubeOptions): Promise<string> {
  // `rehypeYouTube` only acts on `<youtube>` elements. Skip the rehype
  // round-trip entirely when the document has none. The marker is a loose
  // superset of the element name so a real `<youtube>` is never skipped.
  if (!/<youtube/i.test(html)) {
    return html;
  }

  const mergedOptions = { ...defaultOptions, ...options };

  const result = await unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeYouTube, mergedOptions)
    .use(rehypeStringify)
    .process(html);

  return String(result);
}
