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
  /** Directory where avatars, photos, and videos are written. @default "public/ox-content/twitter" */
  mediaOutputDir?: string;
  /** Public URL prefix for downloaded media. @default "/ox-content/twitter" */
  mediaPublicPath?: string;
  /** Download MP4 video and animated GIF assets at build time. @default false */
  downloadVideo?: boolean;
  /** Maximum video size in bytes. Oversized assets are skipped. @default 8388608 */
  maxVideoBytes?: number;
  /** Fetched-card chrome. `"full"` matches sveltweet / react-tweet. @default "compact" */
  appearance?: TweetAppearance;
  /**
   * IANA timezone for full-card timestamps.
   * Invalid values fall back to UTC so build output stays deterministic.
   * @default "UTC"
   */
  timeZone?: string;
}

export interface ResolvedTwitterEmbedOptions {
  fetch: boolean;
  lang: string;
  timeout: number;
  cache: boolean;
  cacheDir: string;
  mediaOutputDir: string;
  mediaPublicPath: string;
  downloadVideo: boolean;
  maxVideoBytes: number;
  appearance: TweetAppearance;
  timeZone: string;
}

export type TweetAppearance = "compact" | "full";

export interface TweetEntity {
  url: string;
  expanded_url?: string;
  display_url?: string;
  indices?: [number, number];
}

export interface TweetVideoVariant {
  content_type?: string;
  url?: string;
  bitrate?: number;
}

export interface TweetMedia extends TweetEntity {
  media_url_https?: string;
  ext_alt_text?: string;
  type?: string;
  original_info?: { width?: number; height?: number };
  video_info?: { variants?: TweetVideoVariant[] };
}

export interface TweetIndexedEntity {
  indices?: [number, number];
}

export interface TweetHashtagEntity extends TweetIndexedEntity {
  text?: string;
}

export interface TweetMentionEntity extends TweetIndexedEntity {
  screen_name?: string;
}

export interface TweetSymbolEntity extends TweetIndexedEntity {
  text?: string;
}

export interface TweetUser {
  name: string;
  screen_name: string;
  profile_image_url_https?: string;
  verified?: boolean;
  is_blue_verified?: boolean;
  verified_type?: string;
}

/** Root or quoted post body. Nested `quoted_tweet` is stripped during normalize. */
export interface TweetBodyData {
  text: string;
  id_str?: string;
  display_text_range?: [number, number];
  entities?: {
    urls?: TweetEntity[];
    media?: TweetMedia[];
    hashtags?: TweetHashtagEntity[];
    user_mentions?: TweetMentionEntity[];
    symbols?: TweetSymbolEntity[];
  };
  mediaDetails?: TweetMedia[];
  user: TweetUser;
  created_at?: string;
}

export interface TweetData extends TweetBodyData {
  quoted_tweet?: TweetBodyData;
  in_reply_to_screen_name?: string;
  in_reply_to_status_id_str?: string;
  favorite_count?: number;
  conversation_count?: number;
  retweet_count?: number;
  repost_count?: number;
  quote_count?: number;
  view_count?: number | string;
  views_count?: number | string;
  impression_count?: number | string;
}

export interface TweetReference {
  id: string;
  url: string;
}

export type TweetMediaKind = "photo" | "video" | "animated_gif";

export interface TweetMediaAsset {
  kind: TweetMediaKind;
  src?: string;
  poster?: string;
  alt?: string;
  width?: number;
  height?: number;
}

export interface TweetAssets {
  avatar?: string;
  media: TweetMediaAsset[];
  quoted?: TweetAssets;
}
