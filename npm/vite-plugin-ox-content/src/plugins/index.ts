/**
 * ox-content Built-in Plugins
 *
 * All plugins are designed with No-JavaScript-First principle.
 * They generate static HTML at build time and require no client-side JS.
 */

import type { GitHubOptions } from "./github";

export { transformTabs, generateTabsCSS, resetTabGroupCounter } from "./tabs";

export { transformYouTube, extractVideoId, type YouTubeOptions } from "./youtube";

export {
  transformGitHub,
  fetchRepoData,
  collectGitHubRepos,
  prefetchGitHubRepos,
  type GitHubRepoData,
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
  youtube?: boolean;
  github?: boolean | GitHubOptions;
  ogp?: boolean;
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
    youtube = true,
    github = true,
    ogp = true,
    mermaid = true,
    githubToken,
  } = options;

  let result = html;

  // Order matters: process in dependency order

  // 1. Tabs (no external dependencies)
  if (tabs) {
    const { transformTabs } = await import("./tabs");
    result = await transformTabs(result);
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
  if (ogp) {
    const { transformOgp } = await import("./ogp");
    result = await transformOgp(result);
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

  return result;
}
