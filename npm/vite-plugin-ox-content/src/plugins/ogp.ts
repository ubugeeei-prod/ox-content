/**
 * OGP Card Plugin - Link card embedding
 *
 * Transforms <OgCard> components into static link preview cards
 * by fetching OGP metadata at build time.
 */

import { unified } from "unified";
import rehypeParse from "rehype-parse";
import rehypeStringify from "rehype-stringify";
import type { Root, Element } from "hast";

export interface OgpData {
  url: string;
  title: string;
  description?: string;
  image?: string;
  siteName?: string;
  favicon?: string;
}

export interface OgpOptions {
  /**
   * Request timeout in milliseconds.
   * @default 10000
   */
  timeout?: number;

  /**
   * Cache fetched Open Graph metadata in memory for the current process.
   * @default true
   */
  cache?: boolean;

  /**
   * Cache TTL in milliseconds.
   * @default 3600000
   */
  cacheTTL?: number;

  /**
   * User agent sent with metadata fetch requests.
   * @default 'ox-content-ogp-bot/1.0 (compatible; +https://github.com/ubugeeei-prod/ox-content)'
   */
  userAgent?: string;
}

const defaultOptions: Required<OgpOptions> = {
  timeout: 10000,
  cache: true,
  cacheTTL: 3600000,
  userAgent: "ox-content-ogp-bot/1.0 (compatible; +https://github.com/ubugeeei-prod/ox-content)",
};

// Simple in-memory cache
const ogpCache = new Map<string, { data: OgpData; timestamp: number }>();

function isPrivateIPv4(hostname: string): boolean {
  const parts = hostname.split(".").map(Number);
  if (
    parts.length !== 4 ||
    parts.some((part) => !Number.isInteger(part) || part < 0 || part > 255)
  ) {
    return false;
  }
  const [a, b] = parts;
  return (
    a === 10 ||
    a === 127 ||
    a === 0 ||
    (a === 172 && b >= 16 && b <= 31) ||
    (a === 192 && b === 168) ||
    (a === 169 && b === 254)
  );
}

export function isSafeOgpUrl(value: string): boolean {
  try {
    const url = new URL(value);
    const host = url.hostname.toLowerCase();
    const ipv6 = host.replace(/^\[|\]$/g, "");
    if (url.protocol !== "http:" && url.protocol !== "https:") return false;
    if (host === "localhost" || host.endsWith(".localhost")) return false;
    if (
      ipv6.includes(":") &&
      (ipv6 === "::1" || ipv6.startsWith("fc") || ipv6.startsWith("fd") || ipv6.startsWith("fe80"))
    )
      return false;
    return !isPrivateIPv4(host);
  } catch {
    return false;
  }
}

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
 * Extract domain from URL.
 */
function extractDomain(url: string): string {
  try {
    const urlObj = new URL(url);
    return urlObj.hostname;
  } catch {
    return url;
  }
}

/**
 * Get favicon URL for a domain.
 */
function getFaviconUrl(url: string): string {
  try {
    const urlObj = new URL(url);
    // Use Google's favicon service as fallback
    return `https://www.google.com/s2/favicons?domain=${urlObj.hostname}&sz=32`;
  } catch {
    return "";
  }
}

/**
 * Parse OGP metadata from HTML.
 */
function parseOgpFromHtml(html: string, url: string): OgpData {
  const result: OgpData = {
    url,
    title: "",
  };

  // Extract title
  const titleMatch = html.match(/<title[^>]*>([^<]+)<\/title>/i);
  const ogTitleMatch =
    html.match(/<meta[^>]*property=["']og:title["'][^>]*content=["']([^"']+)["']/i) ||
    html.match(/<meta[^>]*content=["']([^"']+)["'][^>]*property=["']og:title["']/i);

  result.title = ogTitleMatch?.[1] || titleMatch?.[1] || extractDomain(url);

  // Extract description
  const descMatch =
    html.match(/<meta[^>]*property=["']og:description["'][^>]*content=["']([^"']+)["']/i) ||
    html.match(/<meta[^>]*content=["']([^"']+)["'][^>]*property=["']og:description["']/i) ||
    html.match(/<meta[^>]*name=["']description["'][^>]*content=["']([^"']+)["']/i) ||
    html.match(/<meta[^>]*content=["']([^"']+)["'][^>]*name=["']description["']/i);

  if (descMatch) {
    result.description = descMatch[1];
  }

  // Extract image
  const imageMatch =
    html.match(/<meta[^>]*property=["']og:image["'][^>]*content=["']([^"']+)["']/i) ||
    html.match(/<meta[^>]*content=["']([^"']+)["'][^>]*property=["']og:image["']/i);

  if (imageMatch) {
    let imageUrl = imageMatch[1];
    // Handle relative URLs
    if (imageUrl.startsWith("/")) {
      try {
        const urlObj = new URL(url);
        imageUrl = `${urlObj.protocol}//${urlObj.host}${imageUrl}`;
      } catch {
        // Keep as is
      }
    }
    result.image = imageUrl;
  }

  // Extract site name
  const siteNameMatch =
    html.match(/<meta[^>]*property=["']og:site_name["'][^>]*content=["']([^"']+)["']/i) ||
    html.match(/<meta[^>]*content=["']([^"']+)["'][^>]*property=["']og:site_name["']/i);

  if (siteNameMatch) {
    result.siteName = siteNameMatch[1];
  }

  // Get favicon
  result.favicon = getFaviconUrl(url);

  return result;
}

/**
 * Fetch OGP data for a URL.
 */
export async function fetchOgpData(
  url: string,
  options: Required<OgpOptions>,
): Promise<OgpData | null> {
  if (!isSafeOgpUrl(url)) {
    return null;
  }

  // Check cache
  if (options.cache) {
    const cached = ogpCache.get(url);
    if (cached && Date.now() - cached.timestamp < options.cacheTTL) {
      return cached.data;
    }
  }

  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), options.timeout);

    const response = await fetch(url, {
      headers: {
        "User-Agent": options.userAgent,
        Accept: "text/html,application/xhtml+xml",
      },
      signal: controller.signal,
    });

    clearTimeout(timeoutId);

    if (!response.ok) {
      console.warn(`Failed to fetch OGP for ${url}: ${response.status}`);
      return null;
    }

    const html = await response.text();
    const data = parseOgpFromHtml(html, url);

    // Cache the result
    if (options.cache) {
      ogpCache.set(url, { data, timestamp: Date.now() });
    }

    return data;
  } catch (error) {
    if (error instanceof Error && error.name === "AbortError") {
      console.warn(`Timeout fetching OGP for ${url}`);
    } else {
      console.warn(`Error fetching OGP for ${url}:`, error);
    }
    return null;
  }
}

/**
 * Create OGP card element.
 */
function createOgpCard(data: OgpData): Element {
  const children: Element["children"] = [];

  // Content section
  const contentChildren: Element["children"] = [];

  // Title
  contentChildren.push({
    type: "element",
    tagName: "div",
    properties: { className: ["ox-ogp-title"] },
    children: [{ type: "text", value: data.title }],
  });

  // Description
  if (data.description) {
    contentChildren.push({
      type: "element",
      tagName: "div",
      properties: { className: ["ox-ogp-description"] },
      children: [{ type: "text", value: data.description }],
    });
  }

  // Meta (favicon + domain)
  const metaChildren: Element["children"] = [];

  if (data.favicon) {
    metaChildren.push({
      type: "element",
      tagName: "img",
      properties: {
        className: ["ox-ogp-favicon"],
        src: data.favicon,
        alt: "",
        loading: "lazy",
      },
      children: [],
    });
  }

  metaChildren.push({
    type: "element",
    tagName: "span",
    properties: { className: ["ox-ogp-domain"] },
    children: [{ type: "text", value: data.siteName || extractDomain(data.url) }],
  });

  contentChildren.push({
    type: "element",
    tagName: "div",
    properties: { className: ["ox-ogp-meta"] },
    children: metaChildren,
  });

  children.push({
    type: "element",
    tagName: "div",
    properties: { className: ["ox-ogp-content"] },
    children: contentChildren,
  });

  // Image
  if (data.image) {
    children.push({
      type: "element",
      tagName: "img",
      properties: {
        className: ["ox-ogp-image"],
        src: data.image,
        alt: "",
        loading: "lazy",
      },
      children: [],
    });
  }

  return {
    type: "element",
    tagName: "a",
    properties: {
      className: ["ox-ogp-card"],
      href: isSafeOgpUrl(data.url) ? data.url : "#",
      target: "_blank",
      rel: "noopener noreferrer",
    },
    children,
  };
}

/**
 * Create fallback element when OGP data is unavailable.
 */
function createFallbackCard(url: string): Element {
  return {
    type: "element",
    tagName: "a",
    properties: {
      className: ["ox-ogp-simple"],
      href: isSafeOgpUrl(url) ? url : "#",
      target: "_blank",
      rel: "noopener noreferrer",
    },
    children: [
      {
        type: "element",
        tagName: "svg",
        properties: {
          viewBox: "0 0 24 24",
          fill: "none",
          stroke: "currentColor",
          "stroke-width": "2",
        },
        children: [
          {
            type: "element",
            tagName: "path",
            properties: {
              d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6M15 3h6v6M10 14L21 3",
            },
            children: [],
          },
        ],
      },
      { type: "text", value: extractDomain(url) },
    ],
  };
}

/**
 * Collect all OGP URLs from HTML for pre-fetching.
 */
export async function collectOgpUrls(html: string): Promise<string[]> {
  const urls: string[] = [];
  const urlPattern = /<ogcard[^>]*\s+url=["']([^"']+)["']/gi;

  let match;
  while ((match = urlPattern.exec(html)) !== null) {
    if (isSafeOgpUrl(match[1])) {
      urls.push(match[1]);
    }
  }

  return urls;
}

/**
 * Pre-fetch all OGP data.
 */
export async function prefetchOgpData(
  urls: string[],
  options?: OgpOptions,
): Promise<Map<string, OgpData | null>> {
  const mergedOptions = { ...defaultOptions, ...options };
  const results = new Map<string, OgpData | null>();

  await Promise.all(
    urls.map(async (url) => {
      const data = await fetchOgpData(url, mergedOptions);
      results.set(url, data);
    }),
  );

  return results;
}

/**
 * Rehype plugin to transform OgCard components.
 */
function rehypeOgp(ogpDataMap: Map<string, OgpData | null>) {
  return (tree: Root) => {
    const visit = (node: Root | Element) => {
      if ("children" in node) {
        for (let i = 0; i < node.children.length; i++) {
          const child = node.children[i];

          if (child.type === "element") {
            // Check for <OgCard> component
            if (child.tagName.toLowerCase() === "ogcard") {
              const url = getAttribute(child, "url");

              if (url) {
                const ogpData = ogpDataMap.get(url);
                const cardElement = ogpData ? createOgpCard(ogpData) : createFallbackCard(url);
                node.children[i] = cardElement;
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
 * Transform OgCard components in HTML.
 */
export async function transformOgp(
  html: string,
  ogpDataMap?: Map<string, OgpData | null>,
  options?: OgpOptions,
): Promise<string> {
  // If no pre-fetched data, collect and fetch
  let dataMap = ogpDataMap;
  if (!dataMap) {
    const urls = await collectOgpUrls(html);
    dataMap = await prefetchOgpData(urls, options);
  }

  const result = await unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeOgp, dataMap)
    .use(rehypeStringify)
    .process(html);

  return String(result);
}
