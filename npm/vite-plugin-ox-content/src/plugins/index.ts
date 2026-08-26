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
import {
  createSyndicationToken,
  parseTweetReference,
  type TweetData,
  type TwitterEmbedOptions,
} from "./twitter";
import {
  transformGitHub,
  fetchRepoData,
  fetchGitHubSource,
  collectGitHubRepos,
  collectGitHubSources,
  prefetchGitHubRepos,
  prefetchGitHubSources,
  parseGitHubPermalink,
  parseGitHubLineRange,
  type GitHubRepoData,
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
import { transformMermaidStatic, mermaidClientScript, type MermaidOptions } from "./mermaid";
import { normalizeBlockEmbedParagraphs } from "./block-structure";

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
  fetchRepoData,
  fetchGitHubSource,
  collectGitHubRepos,
  collectGitHubSources,
  prefetchGitHubRepos,
  prefetchGitHubSources,
  parseGitHubPermalink,
  parseGitHubLineRange,
  transformOgp,
  fetchOgpData,
  collectOgpUrls,
  prefetchOgpData,
  transformMermaidStatic,
  mermaidClientScript,
  normalizeBlockEmbedParagraphs,
};

export type {
  PmOptions,
  YouTubeOptions,
  MediaEmbedOptions,
  TweetData,
  TwitterEmbedOptions,
  GitHubRepoData,
  GitHubSourceCommit,
  GitHubSourceData,
  GitHubSourceRef,
  GitHubLineRange,
  GitHubOptions,
  OgpData,
  OgpOptions,
  MermaidOptions,
};

const SELF_CLOSING_EMBED_TAG =
  /<(GitHub|OgCard|Tweet|XPost|Bluesky|Spotify|StackBlitz|WebContainer|YouTube)((?:[^>"']|"[^"]*"|'[^']*')*?)\s*\/>/gi;

/**
 * Custom embed tags are not HTML void elements, so a self-closing authoring
 * form like `<GitHub ... />` reaches the HTML re-parsers (syntax highlighting,
 * embed transforms) as an unclosed element that swallows the rest of the
 * document. Normalize to an explicit open/close pair before any rehype pass
 * runs.
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
export interface TransformAllOptions {
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
  spotify?: boolean;
  stackBlitz?: boolean;
  twitter?: boolean | TwitterEmbedOptions;
  bluesky?: boolean;
  webContainer?: boolean;
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
    stackBlitz = false,
    twitter = false,
    bluesky = false,
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

  const mediaOptions = { spotify, stackBlitz, twitter, bluesky, webContainer };
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
    spotify?: boolean;
    stackBlitz?: boolean;
    twitter?: boolean | TwitterEmbedOptions;
    bluesky?: boolean;
    webContainer?: boolean;
  },
): Promise<string> {
  let result = await normalizeBlockEmbedParagraphs(normalizeSelfClosingEmbeds(html));

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
    stackBlitz: options.stackBlitz,
    twitter: options.twitter,
    bluesky: options.bluesky,
    webContainer: options.webContainer,
  };
  if (Object.values(mediaOptions).some(Boolean)) {
    result = await transformMediaEmbeds(result, mediaOptions);
  }

  return normalizeBlockEmbedParagraphs(result);
}
