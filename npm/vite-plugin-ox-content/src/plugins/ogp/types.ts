import path from "node:path";

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
   * Persistent disk cache also requires this to be enabled.
   * @default true
   */
  cache?: boolean;

  /**
   * Cache TTL in milliseconds. Fresh memory and disk entries skip the network.
   * @default 3600000
   */
  cacheTTL?: number;

  /**
   * Persist successful and negative cache entries to disk across builds.
   * Off by default so existing sites do not write a cache directory.
   * @default false
   */
  persistCache?: boolean;

  /**
   * Directory used for the persistent metadata cache when `persistCache` is on.
   * @default ".cache/ox-content/ogp"
   */
  cacheDir?: string;

  /**
   * Re-fetch metadata even when a fresh cache entry exists.
   * @default false
   */
  refresh?: boolean;

  /**
   * User agent sent with metadata fetch requests.
   * @default 'ox-content-ogp-bot/1.0 (compatible; +https://github.com/ubugeeei-prod/ox-content)'
   */
  userAgent?: string;
}

export interface ResolvedOgpOptions {
  timeout: number;
  cache: boolean;
  cacheTTL: number;
  persistCache: boolean;
  cacheDir: string;
  refresh: boolean;
  userAgent: string;
}

export const DEFAULT_OGP_CACHE_DIR = ".cache/ox-content/ogp";

export const DEFAULT_OGP_USER_AGENT =
  "ox-content-ogp-bot/1.0 (compatible; +https://github.com/ubugeeei-prod/ox-content)";

export function resolveOgpOptions(options: OgpOptions = {}): ResolvedOgpOptions {
  return {
    timeout: options.timeout ?? 10000,
    cache: options.cache ?? true,
    cacheTTL: options.cacheTTL ?? 3_600_000,
    persistCache: options.persistCache ?? false,
    cacheDir: path.resolve(options.cacheDir ?? DEFAULT_OGP_CACHE_DIR),
    refresh: options.refresh ?? false,
    userAgent: options.userAgent ?? DEFAULT_OGP_USER_AGENT,
  };
}
