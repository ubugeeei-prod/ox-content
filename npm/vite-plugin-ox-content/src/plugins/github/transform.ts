import type { Element, Root } from "hast";
import rehypeParse from "rehype-parse";
import rehypeStringify from "rehype-stringify";
import { unified } from "unified";
import { prefetchGitHubRepos, prefetchGitHubSources } from "./api";
import {
  attributesFromElement,
  collectGitHubRepos,
  collectGitHubSources,
  sourceRefFromAttributes,
} from "./attributes";
import { createFallbackCard } from "./fallback-card";
import { createGitHubCard } from "./repo-card";
import { sourceKey } from "./source";
import { createGitHubSourceCard } from "./source-card";
import {
  defaultOptions,
  type GitHubOptions,
  type GitHubRepoData,
  type GitHubSourceData,
} from "./types";

/**
 * Rehype plugin to transform GitHub components.
 */
function rehypeGitHub(
  repoDataMap: Map<string, GitHubRepoData | null>,
  sourceDataMap: Map<string, GitHubSourceData | null>,
  options: Required<GitHubOptions>,
) {
  return (tree: Root) => {
    const visit = (node: Root | Element) => {
      if ("children" in node) {
        for (let i = 0; i < node.children.length; i++) {
          const child = node.children[i];

          if (child.type !== "element") {
            continue;
          }

          if (child.tagName.toLowerCase() !== "github") {
            visit(child);
            continue;
          }

          const attrs = attributesFromElement(child);
          const source = sourceRefFromAttributes(attrs);

          if (source) {
            const sourceData = sourceDataMap.get(sourceKey(source));
            node.children[i] = sourceData
              ? createGitHubSourceCard(sourceData, source.lines, options)
              : createFallbackCard(source.permalink);
            continue;
          }

          const repo = attrs.repo;
          if (repo) {
            const repoData = repoDataMap.get(repo);
            node.children[i] = repoData ? createGitHubCard(repoData) : createFallbackCard(repo);
          }
        }
      }
    };

    visit(tree);
  };
}

/**
 * Transform GitHub components in HTML.
 */
export async function transformGitHub(
  html: string,
  repoDataMap?: Map<string, GitHubRepoData | null>,
  options?: GitHubOptions,
): Promise<string> {
  const mergedOptions = { ...defaultOptions, ...options };
  let dataMap = repoDataMap;
  if (!dataMap) {
    const repos = await collectGitHubRepos(html);
    dataMap = await prefetchGitHubRepos(repos, mergedOptions);
  }
  const sources = await collectGitHubSources(html);
  const sourceDataMap = await prefetchGitHubSources(sources, mergedOptions);

  const result = await unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeGitHub, dataMap, sourceDataMap, mergedOptions)
    .use(rehypeStringify)
    .process(html);

  return String(result);
}
