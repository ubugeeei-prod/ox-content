export interface TwitterEmbedOptions {
  /** Fetch the post body, author, and media from X at build time. */
  fetch?: boolean;
  /** Language sent to the syndication endpoint. @default "en" */
  lang?: string;
  /** Request timeout in milliseconds. @default 10000 */
  timeout?: number;
  /** Cache syndication responses in memory and on disk. @default true */
  cache?: boolean;
  /** Directory used for the persistent metadata cache. @default ".cache/ox-content/twitter" */
  cacheDir?: string;
  /** Directory where avatars and photos are written. @default "public/ox-content/twitter" */
  mediaOutputDir?: string;
  /** Public URL prefix for downloaded media. @default "/ox-content/twitter" */
  mediaPublicPath?: string;
}

export interface ResolvedTwitterEmbedOptions {
  fetch: boolean;
  lang: string;
  timeout: number;
  cache: boolean;
  cacheDir: string;
  mediaOutputDir: string;
  mediaPublicPath: string;
}

export interface TweetEntity {
  url: string;
  expanded_url?: string;
  display_url?: string;
  indices?: [number, number];
}

export interface TweetMedia extends TweetEntity {
  media_url_https?: string;
  ext_alt_text?: string;
  type?: string;
  original_info?: { width?: number; height?: number };
}

export interface TweetData {
  text: string;
  display_text_range?: [number, number];
  entities?: { urls?: TweetEntity[]; media?: TweetMedia[] };
  mediaDetails?: TweetMedia[];
  user: {
    name: string;
    screen_name: string;
    profile_image_url_https?: string;
  };
  created_at?: string;
}

export interface TweetReference {
  id: string;
  url: string;
}

export interface TweetAssets {
  avatar?: string;
  media: Array<{ src: string; alt?: string; width?: number; height?: number }>;
}
