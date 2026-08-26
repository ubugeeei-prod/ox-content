import { importNapiModule } from "../napi";
import {
  enrichProviderArticleEmbeds,
  normalizeProviderArticleOptions,
  type ProviderArticleEmbedOptions,
} from "./provider-articles";
import { enrichSpeakerDeckEmbeds } from "./speaker-deck";
import { transformRedditEmbeds, type RedditEmbedOptions } from "./reddit";
import { transformFetchedTweets } from "./twitter";
import type { TwitterEmbedOptions } from "./twitter";

export interface MediaEmbedOptions {
  /**
   * Render `<Spotify>` embeds.
   * @default false
   */
  spotify?: boolean;

  /**
   * Render `<AppleMusic>` embeds.
   * @default false
   */
  appleMusic?: boolean;

  /**
   * Render `<SpeakerDeck>` embeds.
   * @default false
   */
  speakerDeck?: boolean;

  /**
   * Render `<Audio>` native players.
   * @default false
   */
  audio?: boolean;

  /**
   * Render `<Video>` native players.
   * @default false
   */
  video?: boolean;

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
   * Render `<Reddit>` static cards with build-time metadata fetch.
   * @default false
   */
  reddit?: boolean | RedditEmbedOptions;

  /**
   * Render `<Bluesky>` static cards.
   * @default false
   */
  bluesky?: boolean;

  /**
   * Render `<GoogleMaps>` static place cards.
   * @default false
   */
  googleMaps?: boolean;

  /**
   * Render `<Qiita>` static article cards.
   * Pass `{ fetch: false }` to skip metadata fetching and render a link-only card.
   * @default false
   */
  qiita?: boolean | ProviderArticleEmbedOptions;

  /**
   * Render `<Zenn>` static article cards.
   * Pass `{ fetch: false }` to skip metadata fetching and render a link-only card.
   * @default false
   */
  zenn?: boolean | ProviderArticleEmbedOptions;

  /**
   * Render `<Discord>` static invite/message cards.
   * @default false
   */
  discord?: boolean;

  /**
   * Render `<Fediverse>`, `<Mastodon>`, `<Misskey>`, and `<Mixi2>` static cards.
   * @default false
   */
  fediverse?: boolean;

  /**
   * Render `<Facebook>` static post cards.
   * @default false
   */
  facebook?: boolean;

  /**
   * Render `<Threads>` static post cards.
   * @default false
   */
  threads?: boolean;

  /**
   * Render `<Instagram>` static post cards.
   * @default false
   */
  instagram?: boolean;

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
  if (options.reddit) {
    result = await transformRedditEmbeds(
      result,
      typeof options.reddit === "object" ? options.reddit : {},
    );
  }
  if (options.speakerDeck) {
    result = await enrichSpeakerDeckEmbeds(result);
  }
  if (options.qiita || options.zenn) {
    result = await enrichProviderArticleEmbeds(result, {
      qiita: normalizeProviderArticleOptions(options.qiita),
      zenn: normalizeProviderArticleOptions(options.zenn),
    });
  }
  if (!hasMediaMarker(result)) return result;

  const mod = await importNapiModule();
  return mod.transformMediaEmbeds(result, {
    spotify: options.spotify,
    appleMusic: options.appleMusic,
    speakerDeck: options.speakerDeck,
    audio: options.audio,
    video: options.video,
    stackBlitz: options.stackBlitz,
    twitter: Boolean(options.twitter),
    bluesky: options.bluesky,
    googleMaps: options.googleMaps,
    qiita: Boolean(options.qiita),
    zenn: Boolean(options.zenn),
    discord: options.discord,
    fediverse: options.fediverse,
    facebook: options.facebook,
    threads: options.threads,
    instagram: options.instagram,
    webContainer: options.webContainer,
  });
}

function hasEnabledMediaEmbed(options: MediaEmbedOptions): boolean {
  return Boolean(
    options.spotify ||
    options.appleMusic ||
    options.speakerDeck ||
    options.audio ||
    options.video ||
    options.stackBlitz ||
    options.twitter ||
    options.reddit ||
    options.bluesky ||
    options.googleMaps ||
    options.qiita ||
    options.zenn ||
    options.discord ||
    options.fediverse ||
    options.facebook ||
    options.threads ||
    options.instagram ||
    options.webContainer,
  );
}

function hasMediaMarker(html: string): boolean {
  return (
    /<(spotify|applemusic|speakerdeck|stackblitz|tweet|xpost|reddit|bluesky|googlemaps|qiita|zenn|discord|fediverse|mastodon|misskey|mixi2|facebook|threads|instagram|webcontainer)[\s/>]/i.test(
      html,
    ) || /<(Audio|Video)[\s/>]/.test(html)
  );
}
