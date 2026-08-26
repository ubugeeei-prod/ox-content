import { Buffer } from "node:buffer";
import { gitHubResourceDataFromJson } from "./resource-metadata";
import { resourceKey } from "./resource";
import { inferLanguage, sourceKey, summarizeCommitMessage } from "./source";
import {
  defaultOptions,
  type GitHubOptions,
  type GitHubRepoData,
  type GitHubResourceData,
  type GitHubResourceRef,
  type GitHubSourceCommit,
  type GitHubSourceData,
  type GitHubSourceRef,
} from "./types";
import { encodePath, isSafeGitHubPath, isSafeGitHubRef, isSafeGitHubRepo } from "./validation";

const repoCache = new Map<string, { data: GitHubRepoData; timestamp: number }>();
const sourceCache = new Map<string, { data: GitHubSourceData; timestamp: number }>();
const resourceCache = new Map<string, { data: GitHubResourceData; timestamp: number }>();

interface GitHubContentApiFile {
  type: string;
  encoding?: string;
  content?: string;
  size?: number;
  html_url?: string;
}

interface GitHubCommitApiItem {
  sha?: string;
  html_url?: string;
  commit?: { message?: string };
}

function githubHeaders(options: Required<GitHubOptions>): Record<string, string> {
  const headers: Record<string, string> = {
    Accept: "application/vnd.github.v3+json",
    "User-Agent": "ox-content-github-plugin",
  };

  if (options.token) {
    headers.Authorization = `Bearer ${options.token}`;
  }

  return headers;
}

function githubPublicHeaders(): Record<string, string> {
  return {
    Accept: "application/vnd.github.v3+json",
    "User-Agent": "ox-content-github-plugin",
  };
}

async function fetchSourceCommit(
  source: GitHubSourceRef,
  options: Required<GitHubOptions>,
): Promise<GitHubSourceCommit | undefined> {
  try {
    const apiUrl = `https://api.github.com/repos/${source.repo}/commits?path=${encodeURIComponent(
      source.path,
    )}&sha=${encodeURIComponent(source.ref)}&per_page=1`;
    const response = await fetch(apiUrl, { headers: githubHeaders(options) });
    if (!response.ok) {
      return undefined;
    }

    const items = (await response.json()) as GitHubCommitApiItem[];
    const item = items[0];
    const sha = item?.sha;
    const message = item?.commit?.message ? summarizeCommitMessage(item.commit.message) : "";
    if (!sha || !message) {
      return undefined;
    }

    return {
      sha,
      message,
      html_url: item.html_url ?? `https://github.com/${source.repo}/commit/${sha}`,
    };
  } catch {
    return undefined;
  }
}

export function clearGitHubResourceCache(): void {
  resourceCache.clear();
}

/**
 * Fetch repository data from GitHub API.
 */
export async function fetchRepoData(
  repo: string,
  options: Required<GitHubOptions>,
): Promise<GitHubRepoData | null> {
  if (!isSafeGitHubRepo(repo)) {
    return null;
  }

  if (options.cache) {
    const cached = repoCache.get(repo);
    if (cached && Date.now() - cached.timestamp < options.cacheTTL) {
      return cached.data;
    }
  }

  try {
    const response = await fetch(`https://api.github.com/repos/${repo}`, {
      headers: githubHeaders(options),
    });

    if (!response.ok) {
      console.warn(`Failed to fetch GitHub repo ${repo}: ${response.status}`);
      return null;
    }

    const data = (await response.json()) as GitHubRepoData;
    if (options.cache) {
      repoCache.set(repo, { data, timestamp: Date.now() });
    }

    return data;
  } catch (error) {
    console.warn(`Error fetching GitHub repo ${repo}:`, error);
    return null;
  }
}

/**
 * Fetch source file data from GitHub API.
 */
export async function fetchGitHubSource(
  source: GitHubSourceRef,
  options: Required<GitHubOptions>,
): Promise<GitHubSourceData | null> {
  if (
    !isSafeGitHubRepo(source.repo) ||
    !isSafeGitHubRef(source.ref) ||
    !isSafeGitHubPath(source.path)
  ) {
    return null;
  }

  const key = sourceKey(source);
  if (options.cache) {
    const cached = sourceCache.get(key);
    if (cached && Date.now() - cached.timestamp < options.cacheTTL) {
      return cached.data;
    }
  }

  try {
    const apiUrl = `https://api.github.com/repos/${source.repo}/contents/${encodePath(
      source.path,
    )}?ref=${encodeURIComponent(source.ref)}`;
    const [response, commit] = await Promise.all([
      fetch(apiUrl, { headers: githubHeaders(options) }),
      fetchSourceCommit(source, options),
    ]);

    if (!response.ok) {
      console.warn(`Failed to fetch GitHub source ${source.permalink}: ${response.status}`);
      return null;
    }

    const data = (await response.json()) as GitHubContentApiFile;
    if (
      data.type !== "file" ||
      data.encoding !== "base64" ||
      !data.content ||
      (data.size ?? 0) > options.maxSourceBytes
    ) {
      return null;
    }

    const content = Buffer.from(data.content.replace(/\s/g, ""), "base64").toString("utf8");
    if (Buffer.byteLength(content) > options.maxSourceBytes) {
      return null;
    }

    const sourceData: GitHubSourceData = {
      repo: source.repo,
      ref: source.ref,
      path: source.path,
      permalink: source.permalink,
      content,
      size: data.size ?? Buffer.byteLength(content),
      html_url: data.html_url ?? source.permalink,
      language: inferLanguage(source.path),
      ...(commit ? { commit } : {}),
    };

    if (options.cache) {
      sourceCache.set(key, { data: sourceData, timestamp: Date.now() });
    }

    return sourceData;
  } catch (error) {
    console.warn(`Error fetching GitHub source ${source.permalink}:`, error);
    return null;
  }
}

/**
 * Fetch public GitHub issue, pull request, commit, discussion, or gist metadata.
 */
export async function fetchGitHubResource(
  resource: GitHubResourceRef,
  options: Required<GitHubOptions>,
): Promise<GitHubResourceData | null> {
  const key = resourceKey(resource);
  if (options.cache) {
    const cached = resourceCache.get(key);
    if (cached && Date.now() - cached.timestamp < options.cacheTTL) {
      return cached.data;
    }
  }

  try {
    const response = await fetch(resource.apiUrl, { headers: githubPublicHeaders() });
    if (!response.ok) {
      console.warn(
        `Failed to fetch GitHub ${resource.kind} ${resource.permalink}: ${response.status}`,
      );
      return null;
    }

    const data = gitHubResourceDataFromJson(resource, await response.json());
    if (options.cache && data) {
      resourceCache.set(key, { data, timestamp: Date.now() });
    }
    return data;
  } catch (error) {
    console.warn(`Error fetching GitHub ${resource.kind} ${resource.permalink}:`, error);
    return null;
  }
}

/**
 * Pre-fetch all GitHub repos data.
 */
export async function prefetchGitHubRepos(
  repos: string[],
  options?: GitHubOptions,
): Promise<Map<string, GitHubRepoData | null>> {
  const mergedOptions = { ...defaultOptions, ...options };
  const results = new Map<string, GitHubRepoData | null>();

  await Promise.all(
    Array.from(new Set(repos)).map(async (repo) => {
      const data = await fetchRepoData(repo, mergedOptions);
      results.set(repo, data);
    }),
  );

  return results;
}

/**
 * Pre-fetch all GitHub resource cards.
 */
export async function prefetchGitHubResources(
  resources: GitHubResourceRef[],
  options?: GitHubOptions,
): Promise<Map<string, GitHubResourceData | null>> {
  const mergedOptions = { ...defaultOptions, ...options };
  const results = new Map<string, GitHubResourceData | null>();
  const uniqueResources = Array.from(
    new Map(resources.map((resource) => [resourceKey(resource), resource])).values(),
  );

  await Promise.all(
    uniqueResources.map(async (resource) => {
      const data = await fetchGitHubResource(resource, mergedOptions);
      results.set(resourceKey(resource), data);
    }),
  );

  return results;
}

/**
 * Pre-fetch all GitHub source files.
 */
export async function prefetchGitHubSources(
  sources: GitHubSourceRef[],
  options?: GitHubOptions,
): Promise<Map<string, GitHubSourceData | null>> {
  const mergedOptions = { ...defaultOptions, ...options };
  const results = new Map<string, GitHubSourceData | null>();
  const uniqueSources = Array.from(
    new Map(sources.map((source) => [sourceKey(source), source])).values(),
  );

  await Promise.all(
    uniqueSources.map(async (source) => {
      const data = await fetchGitHubSource(source, mergedOptions);
      results.set(sourceKey(source), data);
    }),
  );

  return results;
}
