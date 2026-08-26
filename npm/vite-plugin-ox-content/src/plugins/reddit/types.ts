export interface RedditEmbedOptions {
  /**
   * Fetch Reddit post metadata at build time.
   * @default true
   */
  fetch?: boolean;

  /**
   * Metadata request timeout in milliseconds.
   * @default 10000
   */
  timeout?: number;

  /**
   * Cache fetched post metadata in memory for the current process.
   * @default true
   */
  cache?: boolean;

  /**
   * Cache TTL in milliseconds. Fresh memory entries skip the network.
   * @default 3600000
   */
  cacheTTL?: number;

  /**
   * User agent sent to Reddit's JSON endpoint.
   * @default 'ox-content-reddit-bot/1.0 (compatible; +https://github.com/ubugeeei-prod/ox-content)'
   */
  userAgent?: string;
}

export interface ResolvedRedditEmbedOptions {
  fetch: boolean;
  timeout: number;
  cache: boolean;
  cacheTTL: number;
  userAgent: string;
}

export interface RedditPostReference {
  url: string;
  id?: string;
  subreddit?: string;
  slug?: string;
  shareId?: string;
  apiUrl?: string;
}

export interface RedditPostImage {
  url: string;
  width?: number;
  height?: number;
}

export interface RedditPostData {
  permalink: string;
  subreddit: string;
  title: string;
  author?: string;
  body?: string;
  score?: number;
  commentCount?: number;
  createdAt?: string;
  originalUrl?: string;
  image?: RedditPostImage;
}

export const DEFAULT_REDDIT_USER_AGENT =
  "ox-content-reddit-bot/1.0 (compatible; +https://github.com/ubugeeei-prod/ox-content)";

export function resolveRedditEmbedOptions(
  options: RedditEmbedOptions = {},
): ResolvedRedditEmbedOptions {
  return {
    fetch: options.fetch ?? true,
    timeout: options.timeout ?? 10000,
    cache: options.cache ?? true,
    cacheTTL: options.cacheTTL ?? 3_600_000,
    userAgent: options.userAgent ?? DEFAULT_REDDIT_USER_AGENT,
  };
}
