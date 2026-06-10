import { importNapiModule } from "../napi";

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
   * Render `<Tweet>` / `<XPost>` static cards.
   * @default false
   */
  twitter?: boolean;

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

  const mod = await importNapiModule();
  return mod.transformMediaEmbeds(html, {
    spotify: options.spotify,
    stackBlitz: options.stackBlitz,
    twitter: options.twitter,
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
