/**
 * ox-content Built-in Plugins
 *
 * All plugins are designed with No-JavaScript-First principle.
 * They generate static HTML at build time and require no client-side JS.
 */

import type { GitHubOptions } from "./github";
import type { MediaEmbedOptions } from "./media";
import type { OgpOptions } from "./ogp";
import type { PmOptions } from "./pm";

export {
  transformTabs,
  generateTabsCSS,
  resetTabGroupCounter,
  getTabGroupCounter,
  setTabGroupCounter,
} from "./tabs";

export { transformPm, type PmOptions } from "./pm";

export { transformYouTube, extractVideoId, type YouTubeOptions } from "./youtube";
export { transformMediaEmbeds, type MediaEmbedOptions } from "./media";

export {
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
  type GitHubSourceData,
  type GitHubSourceRef,
  type GitHubLineRange,
  type GitHubOptions,
} from "./github";

export {
  transformOgp,
  fetchOgpData,
  collectOgpUrls,
  prefetchOgpData,
  type OgpData,
  type OgpOptions,
} from "./ogp";

export { transformMermaidStatic, mermaidClientScript, type MermaidOptions } from "./mermaid";

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
  twitter?: boolean;
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

  let result = html;
  const ogpOptions = openGraph ?? ogp ?? true;

  // Order matters: process in dependency order

  // 1. Tabs (no external dependencies)
  if (tabs) {
    const { transformTabs } = await import("./tabs");
    result = await transformTabs(result);
  }

  // 1b. Package-manager tabs (no external dependencies). Shares the tab-group
  // counter with the tabs transform, so it runs right after it. Syncing is
  // opt-in via `{ pm: { sync: true } }` and off by default.
  if (pm) {
    const { transformPm } = await import("./pm");
    result = await transformPm(result, typeof pm === "object" ? pm : {});
  }

  // 2. YouTube (no external dependencies)
  if (youtube) {
    const { transformYouTube } = await import("./youtube");
    result = await transformYouTube(result);
  }

  // 3. GitHub (requires API calls)
  if (github !== false) {
    const { transformGitHub } = await import("./github");
    const options = typeof github === "object" ? github : {};
    result = await transformGitHub(result, undefined, { token: githubToken, ...options });
  }

  // 4. OGP (requires fetch calls)
  if (ogpOptions !== false) {
    const { transformOgp } = await import("./ogp");
    result = await transformOgp(
      result,
      undefined,
      typeof ogpOptions === "object" ? ogpOptions : {},
    );
  }

  const mediaOptions = { spotify, stackBlitz, twitter, bluesky, webContainer };
  if (Object.values(mediaOptions).some(Boolean)) {
    const { transformMediaEmbeds } = await import("./media");
    result = await transformMediaEmbeds(result, mediaOptions);
  }

  // 5. Mermaid (requires mermaid library)
  if (mermaid) {
    const { transformMermaidStatic } = await import("./mermaid");
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
    twitter?: boolean;
    bluesky?: boolean;
    webContainer?: boolean;
  },
): Promise<string> {
  let result = html;

  if (options.github) {
    const { transformGitHub } = await import("./github");
    result = await transformGitHub(result, undefined, {
      token: process.env.GITHUB_TOKEN,
      ...options.github,
    });
  }

  if (options.openGraph) {
    const { transformOgp } = await import("./ogp");
    result = await transformOgp(result, undefined, options.openGraph);
  }

  if (options.pm) {
    const { transformPm } = await import("./pm");
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
    const { transformMediaEmbeds } = await import("./media");
    result = await transformMediaEmbeds(result, mediaOptions);
  }

  return result;
}
