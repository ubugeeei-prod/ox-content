import type { TwitterEmbedOptions } from "./plugins";
import type { BuiltinPmOptions, OxContentOptions, ResolvedOptions } from "./types";
import { normalizeProviderArticleOptions } from "./plugins/provider-articles";
import { normalizeProviderPackageOptions } from "./plugins/provider-packages";
import { normalizeProviderPlaygroundOptions } from "./plugins/provider-playgrounds";
import { normalizeProviderVideoOptions } from "./plugins/provider-videos";

export function resolveBuiltinEmbedOptions(
  options: OxContentOptions["embeds"],
): ResolvedOptions["embeds"] {
  if (options === false) {
    return {
      github: false,
      openGraph: false,
      pm: false,
      spotify: false,
      appleMusic: false,
      speakerDeck: false,
      audio: false,
      video: false,
      stackBlitz: false,
      twitter: false,
      reddit: false,
      bluesky: false,
      googleMaps: false,
      qiita: false,
      zenn: false,
      packageRegistry: false,
      playgrounds: false,
      vimeo: false,
      twitch: false,
      discord: false,
      fediverse: false,
      facebook: false,
      threads: false,
      instagram: false,
      webContainer: false,
    };
  }

  return {
    github: resolveSingleEmbedOptions(options?.github),
    openGraph: resolveSingleEmbedOptions(options?.openGraph),
    pm: resolvePmOptions(options?.pm),
    spotify: options?.spotify === true,
    appleMusic: options?.appleMusic === true,
    speakerDeck: options?.speakerDeck === true,
    audio: options?.audio === true,
    video: options?.video === true,
    stackBlitz: options?.stackBlitz === true,
    twitter: resolveTwitterEmbedOptions(options?.twitter),
    reddit: options?.reddit === true ? {} : options?.reddit || false,
    bluesky: options?.bluesky === true,
    googleMaps: options?.googleMaps === true,
    qiita: normalizeProviderArticleOptions(options?.qiita),
    zenn: normalizeProviderArticleOptions(options?.zenn),
    packageRegistry: normalizeProviderPackageOptions(options?.packageRegistry),
    playgrounds: normalizeProviderPlaygroundOptions(options?.playgrounds),
    vimeo: normalizeProviderVideoOptions(options?.vimeo),
    twitch: normalizeProviderVideoOptions(options?.twitch),
    discord: options?.discord === true,
    fediverse: options?.fediverse === true,
    facebook: options?.facebook === true,
    threads: options?.threads === true,
    instagram: options?.instagram === true,
    webContainer: options?.webContainer === true,
  };
}

function resolveSingleEmbedOptions<T extends object>(options: boolean | T | undefined): T | false {
  if (options === false) return false;
  if (options === true || options === undefined) return {} as T;
  return options;
}

function resolveTwitterEmbedOptions(
  options: boolean | TwitterEmbedOptions | undefined,
): TwitterEmbedOptions | false {
  if (options === false || options === undefined) return false;
  if (options === true) return {};
  return options;
}

function resolvePmOptions(
  options: boolean | BuiltinPmOptions | undefined,
): BuiltinPmOptions | false {
  if (options === false || options === undefined) return false;
  if (options === true) return {};
  return options;
}
