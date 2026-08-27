import type { OgpData } from "./types";
import { extractDomain, isSafeOgpUrl } from "./url";

const NAMED_ENTITIES: Record<string, string> = {
  amp: "&",
  lt: "<",
  gt: ">",
  quot: '"',
  apos: "'",
  nbsp: " ",
  eacute: "é",
  copy: "©",
  reg: "®",
  trade: "™",
  hellip: "…",
  mdash: "—",
  ndash: "–",
  lsquo: "‘",
  rsquo: "’",
  ldquo: "“",
  rdquo: "”",
};

/**
 * Expand the character references that show up in `<meta>` content.
 *
 * Metadata is markup, so an `og:title` of `Tips & Tricks` reaches us as
 * `Tips &amp; Tricks`. Rendering that verbatim puts the raw entity on the card,
 * and it is the renderer's job — not the author's — to escape the text again.
 */
export function decodeHtmlEntities(value: string): string {
  if (!value.includes("&")) return value;

  return value.replace(/&(#[0-9]+|#[xX][0-9a-fA-F]+|[a-zA-Z][a-zA-Z0-9]*);/g, (match, body) => {
    if (body.startsWith("#")) {
      const code =
        body[1] === "x" || body[1] === "X"
          ? Number.parseInt(body.slice(2), 16)
          : Number.parseInt(body.slice(1), 10);
      if (!Number.isInteger(code) || code <= 0 || code > 0x10ffff) return match;
      try {
        return String.fromCodePoint(code);
      } catch {
        return match;
      }
    }
    return NAMED_ENTITIES[body.toLowerCase()] ?? match;
  });
}

/**
 * Resolve a URL taken from page metadata against the page it came from.
 *
 * Open Graph lets a site write `og:image` as an absolute URL, a
 * protocol-relative one, or a path relative to the document. Only the first
 * form is usable as-is: the other two have to be resolved against the source
 * page, not against whatever docs page ends up embedding the card.
 */
export function resolveMetaUrl(value: string, pageUrl: string): string | undefined {
  const decoded = decodeHtmlEntities(value).trim();
  if (decoded === "") return undefined;

  let resolved: string;
  try {
    resolved = new URL(decoded, pageUrl).href;
  } catch {
    return undefined;
  }

  return isSafeOgpUrl(resolved) ? resolved : undefined;
}

function metaContent(
  html: string,
  attribute: "property" | "name",
  key: string,
): string | undefined {
  const escaped = key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match =
    html.match(
      new RegExp(`<meta[^>]*${attribute}=["']${escaped}["'][^>]*content=["']([^"']*)["']`, "i"),
    ) ??
    html.match(
      new RegExp(`<meta[^>]*content=["']([^"']*)["'][^>]*${attribute}=["']${escaped}["']`, "i"),
    );

  const value = match?.[1] && decodeHtmlEntities(match[1]).trim();
  return value ? value : undefined;
}

function declaredIcon(html: string, pageUrl: string): string | undefined {
  const match =
    html.match(/<link[^>]*rel=["'](?:shortcut )?icon["'][^>]*href=["']([^"']+)["']/i) ??
    html.match(/<link[^>]*href=["']([^"']+)["'][^>]*rel=["'](?:shortcut )?icon["']/i);
  return match ? resolveMetaUrl(match[1], pageUrl) : undefined;
}

/**
 * The site's own icon, never a third-party favicon lookup service.
 *
 * Routing every card through an external service would tell that service which
 * outbound links each documentation page carries, for every reader, and would
 * make cards depend on a host the build never chose.
 */
function faviconUrl(html: string, pageUrl: string): string | undefined {
  return declaredIcon(html, pageUrl) ?? resolveMetaUrl("/favicon.ico", pageUrl);
}

export function parseOgpFromHtml(html: string, url: string): OgpData {
  const titleTag = html.match(/<title[^>]*>([^<]*)<\/title>/i)?.[1];
  const result: OgpData = {
    url,
    title:
      metaContent(html, "property", "og:title") ??
      (titleTag && decodeHtmlEntities(titleTag).trim()) ??
      extractDomain(url),
  };
  if (result.title === "") {
    result.title = extractDomain(url);
  }

  const description =
    metaContent(html, "property", "og:description") ?? metaContent(html, "name", "description");
  if (description) result.description = description;

  const image =
    html.match(/<meta[^>]*property=["']og:image["'][^>]*content=["']([^"']+)["']/i)?.[1] ??
    html.match(/<meta[^>]*content=["']([^"']+)["'][^>]*property=["']og:image["']/i)?.[1];
  const resolvedImage = image && resolveMetaUrl(image, url);
  if (resolvedImage) result.image = resolvedImage;

  const siteName = metaContent(html, "property", "og:site_name");
  if (siteName) result.siteName = siteName;

  const favicon = faviconUrl(html, url);
  if (favicon) result.favicon = favicon;

  return result;
}
