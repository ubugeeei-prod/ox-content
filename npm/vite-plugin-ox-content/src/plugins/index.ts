/**
 * ox-content Built-in Plugins
 *
 * All plugins are designed with No-JavaScript-First principle.
 * They generate static HTML at build time and require no client-side JS.
 */

import {
  transformTabs,
  generateTabsCSS,
  resetTabGroupCounter,
  getTabGroupCounter,
  setTabGroupCounter,
} from "./tabs";
import { transformPm, type PmOptions } from "./pm";
import { transformYouTube, extractVideoId, type YouTubeOptions } from "./youtube";
import { transformMediaEmbeds, type MediaEmbedOptions } from "./media";
import type {
  ProviderArticleEmbedOptions,
  ResolvedProviderArticleEmbedOptions,
} from "./provider-articles";
import type {
  ProviderPackageEmbedOptions,
  ResolvedProviderPackageEmbedOptions,
} from "./provider-packages";
import type {
  ProviderPlaygroundEmbedOptions,
  ResolvedProviderPlaygroundEmbedOptions,
} from "./provider-playgrounds";
import {
  createSyndicationToken,
  parseTweetReference,
  type TweetData,
  type TwitterEmbedOptions,
} from "./twitter";
import {
  transformGitHub,
  clearGitHubResourceCache,
  fetchRepoData,
  fetchGitHubSource,
  fetchGitHubResource,
  collectGitHubResources,
  collectGitHubRepos,
  collectGitHubSources,
  prefetchGitHubResources,
  prefetchGitHubRepos,
  prefetchGitHubSources,
  parseGitHubResourceReference,
  parseGitHubPermalink,
  parseGitHubLineRange,
  type GitHubRepoData,
  type GitHubResourceData,
  type GitHubResourceKind,
  type GitHubResourceRef,
  type GitHubSourceCommit,
  type GitHubSourceData,
  type GitHubSourceRef,
  type GitHubLineRange,
  type GitHubOptions,
} from "./github";
import {
  transformOgp,
  fetchOgpData,
  collectOgpUrls,
  prefetchOgpData,
  type OgpData,
  type OgpOptions,
} from "./ogp";
import {
  transformRedditEmbeds,
  parseRedditPostReference,
  type RedditEmbedOptions,
  type RedditPostData,
  type RedditPostReference,
} from "./reddit";
import { transformMermaidStatic, mermaidClientScript, type MermaidOptions } from "./mermaid";
import { normalizeBlockEmbedParagraphs } from "./block-structure";
import {
  documentLocalComponentNames,
  filterReservedBuiltinComponentNames,
  isReservedBuiltinComponent,
  restoreReservedBuiltinIslands,
  RESERVED_BUILTIN_COMPONENTS,
} from "./embed-transform";

export {
  transformTabs,
  generateTabsCSS,
  resetTabGroupCounter,
  getTabGroupCounter,
  setTabGroupCounter,
  transformPm,
  transformYouTube,
  extractVideoId,
  transformMediaEmbeds,
  createSyndicationToken,
  parseTweetReference,
  transformGitHub,
  clearGitHubResourceCache,
  fetchRepoData,
  fetchGitHubSource,
  fetchGitHubResource,
  collectGitHubResources,
  collectGitHubRepos,
  collectGitHubSources,
  prefetchGitHubResources,
  prefetchGitHubRepos,
  prefetchGitHubSources,
  parseGitHubResourceReference,
  parseGitHubPermalink,
  parseGitHubLineRange,
  transformOgp,
  fetchOgpData,
  collectOgpUrls,
  prefetchOgpData,
  transformRedditEmbeds,
  parseRedditPostReference,
  transformMermaidStatic,
  mermaidClientScript,
  normalizeBlockEmbedParagraphs,
  restoreReservedBuiltinIslands,
  isReservedBuiltinComponent,
  documentLocalComponentNames,
  filterReservedBuiltinComponentNames,
  RESERVED_BUILTIN_COMPONENTS,
};

export type {
  PmOptions,
  YouTubeOptions,
  MediaEmbedOptions,
  TweetData,
  TwitterEmbedOptions,
  GitHubRepoData,
  GitHubResourceData,
  GitHubResourceKind,
  GitHubResourceRef,
  GitHubSourceCommit,
  GitHubSourceData,
  GitHubSourceRef,
  GitHubLineRange,
  GitHubOptions,
  OgpData,
  OgpOptions,
  RedditEmbedOptions,
  RedditPostData,
  RedditPostReference,
  ProviderArticleEmbedOptions,
  ResolvedProviderArticleEmbedOptions,
  ProviderPackageEmbedOptions,
  ResolvedProviderPackageEmbedOptions,
  ProviderPlaygroundEmbedOptions,
  ResolvedProviderPlaygroundEmbedOptions,
  MermaidOptions,
};

const SELF_CLOSING_EMBED_TAG =
  /<(GitHub|OgCard|Tweet|XPost|Reddit|Bluesky|GoogleMaps|Qiita|Zenn|NpmPackage|CratesIo|PyPI|DockerHub|CodePen|JSFiddle|Observable|Discord|Fediverse|Mastodon|Misskey|Mixi2|Facebook|Threads|Instagram|Spotify|AppleMusic|SpeakerDeck|Audio|Video|StackBlitz|WebContainer|YouTube|NotByAI)((?:[^>"']|"[^"]*"|'[^']*')*?)\s*\/>(?:\s*<\/\1\s*>)?/gi;

/**
 * Custom embed tags are not HTML void elements, so a self-closing authoring
 * form like `<GitHub ... />` reaches the HTML re-parsers (syntax highlighting,
 * embed transforms) as an unclosed element that swallows the rest of the
 * document. Normalize to an explicit open/close pair before any rehype pass
 * runs. A leftover `</Tweet>` after `/>` (CommonMark HTML + self-close) is
 * consumed so the closer cannot survive the embed rewrite.
 */
export function normalizeSelfClosingEmbeds(html: string): string {
  return html.replace(SELF_CLOSING_EMBED_TAG, (_match, tag: string, attrs: string) => {
    return `<${tag}${attrs}></${tag}>`;
  });
}

/**
 * Transform all plugin components in HTML.
 * Call this during SSG build to process all plugins at once.
 */
export interface TransformAllOptions extends MediaEmbedOptions {
  tabs?: boolean;
  /**
   * Expand `<pm>` package-manager blocks into install tabs. Pass an object to
   * opt in to synced groups (`{ sync: true }`); syncing is off by default.
   * @default false
   */
  pm?: boolean | PmOptions;
  youtube?: boolean;
  github?: boolean | GitHubOptions;
  ogp?: boolean | OgpOptions;
  openGraph?: boolean | OgpOptions;
  mermaid?: boolean;
  githubToken?: string;
}

/**
 * Transform all enabled plugins in HTML content.
 */
export async function transformAllPlugins(
  html: string,
  options: TransformAllOptions = {},
): Promise<string> {
  const {
    tabs = true,
    pm = false,
    youtube = true,
    github = true,
    ogp,
    openGraph,
    mermaid = true,
    githubToken,
    spotify = false,
    appleMusic = false,
    speakerDeck = false,
    audio = false,
    video = false,
    stackBlitz = false,
    twitter = false,
    reddit = false,
    bluesky = false,
    googleMaps = false,
    qiita = false,
    zenn = false,
    packageRegistry = false,
    playgrounds = false,
    discord = false,
    fediverse = false,
    facebook = false,
    threads = false,
    instagram = false,
    webContainer = false,
  } = options;

  let result = await normalizeBlockEmbedParagraphs(normalizeSelfClosingEmbeds(html));
  const ogpOptions = openGraph ?? ogp ?? true;

  // Order matters: process in dependency order

  // 1. Tabs (no external dependencies)
  if (tabs) {
    result = await transformTabs(result);
  }

  // 1b. Package-manager tabs (no external dependencies). Shares the tab-group
  // counter with the tabs transform, so it runs right after it. Syncing is
  // opt-in via `{ pm: { sync: true } }` and off by default.
  if (pm) {
    result = await transformPm(result, typeof pm === "object" ? pm : {});
  }

  // 2. YouTube (no external dependencies)
  if (youtube) {
    result = await transformYouTube(result);
  }

  // 3. GitHub (requires API calls)
  if (github !== false) {
    const options = typeof github === "object" ? github : {};
    result = await transformGitHub(result, undefined, { token: githubToken, ...options });
  }

  // 4. OGP (requires fetch calls)
  if (ogpOptions !== false) {
    result = await transformOgp(
      result,
      undefined,
      typeof ogpOptions === "object" ? ogpOptions : {},
    );
  }

  const mediaOptions = {
    spotify,
    appleMusic,
    speakerDeck,
    audio,
    video,
    stackBlitz,
    twitter,
    reddit,
    bluesky,
    googleMaps,
    qiita,
    zenn,
    packageRegistry,
    playgrounds,
    discord,
    fediverse,
    facebook,
    threads,
    instagram,
    webContainer,
  };
  if (Object.values(mediaOptions).some(Boolean)) {
    result = await transformMediaEmbeds(result, mediaOptions);
  }

  result = await normalizeBlockEmbedParagraphs(result);

  // 5. Mermaid (requires mermaid library)
  if (mermaid) {
    result = await transformMermaidStatic(result);
  }

  return result;
}

/**
 * Transform built-in embed components in HTML content.
 */
export async function transformBuiltinEmbeds(
  html: string,
  options: {
    github: GitHubOptions | false;
    openGraph: OgpOptions | false;
    pm?: PmOptions | false;
    /**
     * Document-local import bindings. A reserved built-in name in this set
     * stays an MDX island instead of running the first-party embed transform.
     */
    localNames?: Iterable<string>;
  } & MediaEmbedOptions,
): Promise<string> {
  let result = await normalizeBlockEmbedParagraphs(
    restoreReservedBuiltinIslands(normalizeSelfClosingEmbeds(html), options.localNames),
  );

  if (options.github) {
    result = await transformGitHub(result, undefined, {
      token: process.env.GITHUB_TOKEN,
      ...options.github,
    });
  }

  if (options.openGraph) {
    result = await transformOgp(result, undefined, options.openGraph);
  }

  if (options.pm) {
    result = await transformPm(result, typeof options.pm === "object" ? options.pm : {});
  }

  const mediaOptions: MediaEmbedOptions = {
    spotify: options.spotify,
    appleMusic: options.appleMusic,
    speakerDeck: options.speakerDeck,
    audio: options.audio,
    video: options.video,
    stackBlitz: options.stackBlitz,
    twitter: options.twitter,
    reddit: options.reddit,
    bluesky: options.bluesky,
    googleMaps: options.googleMaps,
    qiita: options.qiita,
    zenn: options.zenn,
    packageRegistry: options.packageRegistry,
    playgrounds: options.playgrounds,
    discord: options.discord,
    fediverse: options.fediverse,
    facebook: options.facebook,
    threads: options.threads,
    instagram: options.instagram,
    webContainer: options.webContainer,
  };
  if (Object.values(mediaOptions).some(Boolean)) {
    result = await transformMediaEmbeds(result, mediaOptions);
  }

  return normalizeBlockEmbedParagraphs(result);
}
