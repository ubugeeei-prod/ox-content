import { importNapiModule } from "../napi";
import {
  enrichProviderArticleEmbeds,
  normalizeProviderArticleOptions,
  type ProviderArticleEmbedOptions,
} from "./provider-articles";
import {
  enrichProviderPackageEmbeds,
  normalizeProviderPackageOptions,
  type ProviderPackageEmbedOptions,
} from "./provider-packages";
import {
  enrichProviderPlaygroundEmbeds,
  normalizeProviderPlaygroundOptions,
  type ProviderPlaygroundEmbedOptions,
} from "./provider-playgrounds";
import {
  enrichProviderVideoEmbeds,
  normalizeProviderVideoOptions,
  type ProviderVideoEmbedOptions,
} from "./provider-videos";
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
   * Render `<NpmPackage>`, `<CratesIo>`, `<PyPI>`, and `<DockerHub>` package cards.
   * Pass `{ fetch: false }` to skip metadata fetching and render link-only cards.
   * @default false
   */
  packageRegistry?: boolean | ProviderPackageEmbedOptions;

  /**
   * Render `<CodePen>`, `<JSFiddle>`, and `<Observable>` static playground cards.
   * Pass `{ iframe: true }` to add lazy provider iframe URLs where supported.
   * @default false
   */
  playgrounds?: boolean | ProviderPlaygroundEmbedOptions;

  /**
   * Render `<Vimeo>` static video cards.
   * Pass `{ iframe: true }` to add lazy player iframe URLs.
   * @default false
   */
  vimeo?: boolean | ProviderVideoEmbedOptions;

  /**
   * Render `<Twitch>` static video, clip, and channel cards.
   * Pass `{ iframe: true, parent: "example.com" }` to add Twitch player iframes.
   * @default false
   */
  twitch?: boolean | ProviderVideoEmbedOptions;

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
  if (options.packageRegistry) {
    result = await enrichProviderPackageEmbeds(
      result,
      normalizeProviderPackageOptions(options.packageRegistry),
    );
  }
  if (options.playgrounds) {
    result = await enrichProviderPlaygroundEmbeds(
      result,
      normalizeProviderPlaygroundOptions(options.playgrounds),
    );
  }
  if (options.vimeo || options.twitch) {
    result = await enrichProviderVideoEmbeds(result, {
      vimeo: normalizeProviderVideoOptions(options.vimeo),
      twitch: normalizeProviderVideoOptions(options.twitch),
    });
  }
  if (!hasMediaMarker(result)) return result;

  const mod = await importNapiModule();
  const transformed = mod.transformMediaEmbedsWithDiagnostics(result, {
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
    packageRegistry: Boolean(options.packageRegistry),
    playgrounds: Boolean(options.playgrounds),
    vimeo: Boolean(options.vimeo),
    twitch: Boolean(options.twitch),
    discord: options.discord,
    fediverse: options.fediverse,
    facebook: options.facebook,
    threads: options.threads,
    instagram: options.instagram,
    webContainer: options.webContainer,
  });

  reportRefusedEmbeds(transformed.diagnostics);
  return transformed.html;
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
    options.packageRegistry ||
    options.playgrounds ||
    options.vimeo ||
    options.twitch ||
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
    /<(spotify|applemusic|speakerdeck|stackblitz|tweet|xpost|reddit|bluesky|googlemaps|qiita|zenn|npmpackage|cratesio|pypi|dockerhub|codepen|jsfiddle|observable|vimeo|twitch|discord|fediverse|mastodon|misskey|mixi2|facebook|threads|instagram|webcontainer)[\s/>]/i.test(
      html,
    ) || /<(Audio|Video)[\s/>]/.test(html)
  );
}

/**
 * Warns about tags an enabled provider refused.
 *
 * A refusal is invisible in the output — the tag becomes a plain link, or stays
 * as authored — so a mistyped URL would otherwise ship without a word.
 */
function reportRefusedEmbeds(
  diagnostics: readonly { provider: string; url?: string | null; line: number; fallback: string }[],
): void {
  for (const diagnostic of diagnostics) {
    const target = diagnostic.url ? ` for ${diagnostic.url}` : "";
    const outcome =
      diagnostic.fallback === "linked"
        ? "rendered as a plain link instead"
        : "left as authored markup; it carries no URL safe to link to";
    console.warn(
      `[ox-content:embeds] <${diagnostic.provider}> on line ${diagnostic.line} did not resolve${target}; ${outcome}.`,
    );
  }
}
