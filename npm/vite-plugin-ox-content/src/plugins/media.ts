import { importNapiModule } from "../napi";
import { transformFetchedTweets } from "./twitter";
import type { TwitterEmbedOptions } from "./twitter";

export interface MediaEmbedOptions {
  /**
   * Render `<Spotify>` embeds.
   * @default false
   */
  spotify?: boolean;

  /**
   * Render `<StackBlitz>` embeds.
   * @default false
   */
  stackBlitz?: boolean;

  /**
   * Render `<Tweet>` / `<XPost>` static cards. Pass `{ fetch: true }` to
   * resolve the post content and self-host its media at build time.
   * @default false
   */
  twitter?: boolean | TwitterEmbedOptions;

  /**
   * Render `<Bluesky>` static cards.
   * @default false
   */
  bluesky?: boolean;

  /**
   * Render `<WebContainer>` lazy placeholder blocks.
   * @default false
   */
  webContainer?: boolean;
}

export async function transformMediaEmbeds(
  html: string,
  options: MediaEmbedOptions,
): Promise<string> {
  if (!hasEnabledMediaEmbed(options) || !hasMediaMarker(html)) {
    return html;
  }

  let result = html;
  if (typeof options.twitter === "object") {
    result = await transformFetchedTweets(result, options.twitter);
  }
  if (!hasMediaMarker(result)) return result;

  const mod = await importNapiModule();
  return mod.transformMediaEmbeds(result, {
    spotify: options.spotify,
    stackBlitz: options.stackBlitz,
    twitter: Boolean(options.twitter),
    bluesky: options.bluesky,
    webContainer: options.webContainer,
  });
}

function hasEnabledMediaEmbed(options: MediaEmbedOptions): boolean {
  return Boolean(
    options.spotify ||
    options.stackBlitz ||
    options.twitter ||
    options.bluesky ||
    options.webContainer,
  );
}

function hasMediaMarker(html: string): boolean {
  return /<(spotify|stackblitz|tweet|xpost|bluesky|webcontainer)[\s/>]/i.test(html);
}
