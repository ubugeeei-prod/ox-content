import { importNapiModule } from "../napi";

export interface MediaEmbedOptions {
  spotify?: boolean;
  stackBlitz?: boolean;
  twitter?: boolean;
  bluesky?: boolean;
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
