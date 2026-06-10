/**
 * YouTube Plugin - Privacy-enhanced iframe embedding
 *
 * Transforms <YouTube> components into responsive iframe embeds using
 * youtube-nocookie.com for enhanced privacy.
 *
 * The HTML rewrite is performed in Rust (`transformYoutubeEmbeds` in
 * @ox-content/napi), replacing the previous rehype parse/stringify
 * round-trip. This module keeps the public TS surface and a cheap marker
 * check so pages without a `<youtube>` element never cross the NAPI boundary.
 */

import { importNapiModule } from "../napi";

export interface YouTubeOptions {
  /**
   * Use privacy-enhanced mode (`youtube-nocookie.com`).
   * @default true
   */
  privacyEnhanced?: boolean;

  /**
   * Default iframe aspect ratio.
   * @default '16/9'
   */
  aspectRatio?: string;

  /**
   * Allow fullscreen playback.
   * @default true
   */
  allowFullscreen?: boolean;

  /**
   * Lazy load the iframe.
   * @default true
   */
  lazyLoad?: boolean;
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
 * Transform YouTube components in HTML.
 */
export async function transformYouTube(html: string, options?: YouTubeOptions): Promise<string> {
  // Cheap marker check: skip the NAPI call entirely when there's no
  // `<youtube>` element (the common case). The Rust side guards the same way,
  // but short-circuiting here avoids marshalling the whole document across
  // the boundary.
  if (!/<youtube/i.test(html)) {
    return html;
  }

  const mod = await importNapiModule();
  return mod.transformYoutubeEmbeds(html, options);
}
