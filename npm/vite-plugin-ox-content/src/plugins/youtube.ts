/**
 * YouTube Plugin - Privacy-enhanced iframe embedding
 *
 * Transforms <YouTube> components into responsive iframe embeds using
 * youtube-nocookie.com for enhanced privacy. A digits-only `start` attribute
 * becomes `?start=` on the iframe URL.
 *
 * The HTML rewrite is performed in Rust (`transformYoutubeEmbeds` in
 * @ox-content/napi), replacing the previous rehype parse/stringify
 * round-trip. This module keeps the public TS surface and a cheap marker
 * check so pages without a `<youtube>` element never cross the NAPI boundary.
 */

import { importNapiModule, importNapiModuleSync } from "../napi";

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
 * Extract a YouTube video ID from a bare ID or a watch / share / embed /
 * shorts URL.
 *
 * The rule lives in Rust so this function and the `<youtube>` rewrite below
 * cannot disagree about what counts as a video: `transformYoutubeEmbeds`
 * resolves IDs with the same code.
 */
export function extractVideoId(input: string): string | null {
  return importNapiModuleSync().extractYoutubeVideoId(input);
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
