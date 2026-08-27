/**
 * OGP Card Plugin - Link card embedding
 *
 * Transforms <OgCard> components into static link preview cards
 * by fetching OGP metadata at build time.
 */

export { clearOgpCache } from "./ogp/cache";
export { fetchOgpData } from "./ogp/fetch";
export { decodeHtmlEntities, parseOgpFromHtml } from "./ogp/parse";
export { collectOgpUrls, prefetchOgpData, transformOgp } from "./ogp/transform";
export { resolveOgpOptions } from "./ogp/types";
export type { OgpData, OgpOptions, ResolvedOgpOptions } from "./ogp/types";
export { isSafeOgpUrl, normalizeOgpUrl, ogpCacheKey } from "./ogp/url";
