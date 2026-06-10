/**
 * GitHub Plugin - Repository and source code embedding
 *
 * Transforms <GitHub> components into static repository and source code cards
 * by fetching data from GitHub API at build time.
 */

import { Buffer } from "node:buffer";
import { unified } from "unified";
import rehypeParse from "rehype-parse";
import rehypeStringify from "rehype-stringify";
import type { Root, Element } from "hast";

export interface GitHubRepoData {
  name: string;
  full_name: string;
  description: string | null;
  html_url: string;
  stargazers_count: number;
  forks_count: number;
  language: string | null;
  owner: {
    login: string;
    avatar_url: string;
  };
}

export interface GitHubLineRange {
  start: number;
  end: number;
}

export interface GitHubSourceRef {
  repo: string;
  ref: string;
  path: string;
  permalink: string;
  lines?: GitHubLineRange;
}

export interface GitHubSourceData {
  repo: string;
  ref: string;
  path: string;
  permalink: string;
  content: string;
  size: number;
  html_url: string;
  language: string | null;
}

export interface GitHubOptions {
  /**
   * GitHub API token used for higher rate limits and private repository access.
   * @default ''
   */
  token?: string;

  /**
   * Cache fetched repository and source data in memory for the current process.
   * @default true
   */
  cache?: boolean;

  /**
   * Cache TTL in milliseconds.
   * @default 3600000
   */
  cacheTTL?: number;

  /**
   * Maximum source file size to inline in bytes.
   * @default 200000
   */
  maxSourceBytes?: number;

  /**
   * Maximum source lines to inline when no line range is specified.
   * @default 120
   */
  maxSourceLines?: number;
}

const defaultOptions: Required<GitHubOptions> = {
  token: "",
  cache: true,
  cacheTTL: 3600000,
  maxSourceBytes: 200000,
  maxSourceLines: 120,
};

// Simple in-memory cache
const repoCache = new Map<string, { data: GitHubRepoData; timestamp: number }>();
const sourceCache = new Map<string, { data: GitHubSourceData; timestamp: number }>();
const GITHUB_REPO_RE = /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/;
const GITHUB_COMPONENT_RE = /<github\b([^>]*)>/gi;
const ATTRIBUTE_RE = /([:\w-]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>/]+)))?/g;
const EXTENSION_LANGUAGE_MAP = new Map<string, string>([
  ["cjs", "javascript"],
  ["css", "css"],
  ["go", "go"],
  ["html", "html"],
  ["js", "javascript"],
  ["json", "json"],
  ["jsx", "jsx"],
  ["md", "markdown"],
  ["mdx", "mdx"],
  ["mjs", "javascript"],
  ["py", "python"],
  ["rb", "ruby"],
  ["rs", "rust"],
  ["sh", "shell"],
  ["svelte", "svelte"],
  ["toml", "toml"],
  ["ts", "typescript"],
  ["tsx", "tsx"],
  ["vue", "vue"],
  ["yaml", "yaml"],
  ["yml", "yaml"],
]);

function hasControlChar(value: string): boolean {
  for (let index = 0; index < value.length; index++) {
    const code = value.charCodeAt(index);
    if (code <= 0x1f || code === 0x7f) {
      return true;
    }
  }
  return false;
}

export function isSafeGitHubRepo(repo: string): boolean {
  return (
    GITHUB_REPO_RE.test(repo) && !repo.split("/").some((part) => part === "." || part === "..")
  );
}

function isSafeGitHubRef(ref: string): boolean {
  return Boolean(ref) && !hasControlChar(ref) && !hasUnsafePathSegment(ref);
}

function isSafeGitHubPath(path: string): boolean {
  return Boolean(path) && !hasControlChar(path) && !hasUnsafePathSegment(path);
}

function hasUnsafePathSegment(value: string): boolean {
  return value
    .split("/")
    .some((part) => !part || part === "." || part === ".." || part.includes("\\"));
}

function encodePath(path: string): string {
  return path.split("/").map(encodeURIComponent).join("/");
}

function sourceKey(source: GitHubSourceRef): string {
  return `${source.repo}@${source.ref}:${source.path}`;
}

function formatLineRange(lines: GitHubLineRange): string {
  return lines.start === lines.end ? `L${lines.start}` : `L${lines.start}-L${lines.end}`;
}

export function parseGitHubLineRange(value: string | undefined): GitHubLineRange | undefined {
  if (!value) return undefined;
  const match = value.trim().match(/^#?L?(\d+)(?:-L?(\d+))?$/i);
  if (!match) return undefined;

  const start = Number.parseInt(match[1], 10);
  const end = match[2] ? Number.parseInt(match[2], 10) : start;
  if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end) || start < 1 || end < start) {
    return undefined;
  }

  return { start, end };
}

export function createGitHubPermalink(source: Omit<GitHubSourceRef, "permalink">): string {
  const fragment = source.lines ? `#${formatLineRange(source.lines)}` : "";
  return `https://github.com/${source.repo}/blob/${encodeURIComponent(source.ref)}/${encodePath(
    source.path,
  )}${fragment}`;
}

export function parseGitHubPermalink(value: string): GitHubSourceRef | null {
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    return null;
  }

  if (url.protocol !== "https:" || url.hostname !== "github.com") {
    return null;
  }

  let parts: string[];
  try {
    parts = url.pathname
      .split("/")
      .filter(Boolean)
      .map((part) => decodeURIComponent(part));
  } catch {
    return null;
  }

  if (parts.length < 5 || parts[2] !== "blob") {
    return null;
  }

  const repo = `${parts[0]}/${parts[1]}`;
  const ref = parts[3];
  const path = parts.slice(4).join("/");
  if (!isSafeGitHubRepo(repo) || !isSafeGitHubRef(ref) || !isSafeGitHubPath(path)) {
    return null;
  }

  const lines = parseGitHubLineRange(url.hash);
  const source = { repo, ref, path, lines };
  return {
    ...source,
    permalink: createGitHubPermalink(source),
  };
}

/**
 * Get element attribute value.
 */
function getAttribute(el: Element, name: string): string | undefined {
  const value = el.properties?.[name];
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return value.join(" ");
  return undefined;
}

/**
 * Format number with K/M suffix.
 */
function formatNumber(num: number): string {
  if (num >= 1000000) {
    return `${(num / 1000000).toFixed(1)}M`;
  }
  if (num >= 1000) {
    return `${(num / 1000).toFixed(1)}k`;
  }
  return String(num);
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

  // Check cache
  if (options.cache) {
    const cached = repoCache.get(repo);
    if (cached && Date.now() - cached.timestamp < options.cacheTTL) {
      return cached.data;
    }
  }

  try {
    const headers: Record<string, string> = {
      Accept: "application/vnd.github.v3+json",
      "User-Agent": "ox-content-github-plugin",
    };

    if (options.token) {
      headers.Authorization = `Bearer ${options.token}`;
    }

    const response = await fetch(`https://api.github.com/repos/${repo}`, { headers });

    if (!response.ok) {
      console.warn(`Failed to fetch GitHub repo ${repo}: ${response.status}`);
      return null;
    }

    const data = (await response.json()) as GitHubRepoData;

    // Cache the result
    if (options.cache) {
      repoCache.set(repo, { data, timestamp: Date.now() });
    }

    return data;
  } catch (error) {
    console.warn(`Error fetching GitHub repo ${repo}:`, error);
    return null;
  }
}

interface GitHubContentApiFile {
  type: string;
  encoding?: string;
  content?: string;
  size?: number;
  html_url?: string;
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
    const headers: Record<string, string> = {
      Accept: "application/vnd.github.v3+json",
      "User-Agent": "ox-content-github-plugin",
    };

    if (options.token) {
      headers.Authorization = `Bearer ${options.token}`;
    }

    const apiUrl = `https://api.github.com/repos/${source.repo}/contents/${encodePath(
      source.path,
    )}?ref=${encodeURIComponent(source.ref)}`;
    const response = await fetch(apiUrl, { headers });

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
 * Create GitHub card element from repo data.
 */
function createGitHubCard(repoData: GitHubRepoData): Element {
  const statsChildren: Element["children"] = [];

  // Language
  if (repoData.language) {
    statsChildren.push({
      type: "element",
      tagName: "span",
      properties: { className: ["ox-github-language"] },
      children: [
        {
          type: "element",
          tagName: "span",
          properties: {
            className: ["ox-github-language-color"],
            "data-lang": repoData.language.toLowerCase(),
          },
          children: [],
        },
        { type: "text", value: repoData.language },
      ],
    });
  }

  // Stars
  statsChildren.push({
    type: "element",
    tagName: "span",
    properties: { className: ["ox-github-stat"] },
    children: [
      {
        type: "element",
        tagName: "svg",
        properties: {
          viewBox: "0 0 16 16",
          fill: "currentColor",
        },
        children: [
          {
            type: "element",
            tagName: "path",
            properties: {
              d: "M8 .25a.75.75 0 0 1 .673.418l1.882 3.815 4.21.612a.75.75 0 0 1 .416 1.279l-3.046 2.97.719 4.192a.751.751 0 0 1-1.088.791L8 12.347l-3.766 1.98a.75.75 0 0 1-1.088-.79l.72-4.194L.818 6.374a.75.75 0 0 1 .416-1.28l4.21-.611L7.327.668A.75.75 0 0 1 8 .25Z",
            },
            children: [],
          },
        ],
      },
      { type: "text", value: formatNumber(repoData.stargazers_count) },
    ],
  });

  // Forks
  statsChildren.push({
    type: "element",
    tagName: "span",
    properties: { className: ["ox-github-stat"] },
    children: [
      {
        type: "element",
        tagName: "svg",
        properties: {
          viewBox: "0 0 16 16",
          fill: "currentColor",
        },
        children: [
          {
            type: "element",
            tagName: "path",
            properties: {
              d: "M5 5.372v.878c0 .414.336.75.75.75h4.5a.75.75 0 0 0 .75-.75v-.878a2.25 2.25 0 1 1 1.5 0v.878a2.25 2.25 0 0 1-2.25 2.25h-1.5v2.128a2.251 2.251 0 1 1-1.5 0V8.5h-1.5A2.25 2.25 0 0 1 3.5 6.25v-.878a2.25 2.25 0 1 1 1.5 0ZM5 3.25a.75.75 0 1 0-1.5 0 .75.75 0 0 0 1.5 0Zm6.75.75a.75.75 0 1 0 0-1.5.75.75 0 0 0 0 1.5Zm-3 8.75a.75.75 0 1 0-1.5 0 .75.75 0 0 0 1.5 0Z",
            },
            children: [],
          },
        ],
      },
      { type: "text", value: formatNumber(repoData.forks_count) },
    ],
  });

  return {
    type: "element",
    tagName: "a",
    properties: {
      className: ["ox-github-card"],
      href: repoData.html_url,
      target: "_blank",
      rel: "noopener noreferrer",
    },
    children: [
      // Header
      {
        type: "element",
        tagName: "div",
        properties: { className: ["ox-github-header"] },
        children: [
          {
            type: "element",
            tagName: "svg",
            properties: {
              className: ["ox-github-icon"],
              viewBox: "0 0 16 16",
              fill: "currentColor",
            },
            children: [
              {
                type: "element",
                tagName: "path",
                properties: {
                  d: "M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8ZM5 12.25a.25.25 0 0 1 .25-.25h3.5a.25.25 0 0 1 .25.25v3.25a.25.25 0 0 1-.4.2l-1.45-1.087a.249.249 0 0 0-.3 0L5.4 15.7a.25.25 0 0 1-.4-.2Z",
                },
                children: [],
              },
            ],
          },
          {
            type: "element",
            tagName: "span",
            properties: { className: ["ox-github-repo"] },
            children: [{ type: "text", value: repoData.full_name }],
          },
        ],
      },
      // Description
      ...(repoData.description
        ? [
            {
              type: "element" as const,
              tagName: "p",
              properties: { className: ["ox-github-description"] },
              children: [{ type: "text" as const, value: repoData.description }],
            },
          ]
        : []),
      // Stats
      {
        type: "element",
        tagName: "div",
        properties: { className: ["ox-github-stats"] },
        children: statsChildren,
      },
    ],
  };
}

/**
 * Create fallback element when repo data is unavailable.
 */
function createFallbackCard(repo: string): Element {
  const href = isSafeGitHubRepo(repo) ? `https://github.com/${repo}` : "#";
  return {
    type: "element",
    tagName: "a",
    properties: {
      className: ["ox-github-card", "error"],
      href,
      target: "_blank",
      rel: "noopener noreferrer",
    },
    children: [
      {
        type: "element",
        tagName: "div",
        properties: { className: ["ox-github-header"] },
        children: [
          {
            type: "element",
            tagName: "svg",
            properties: {
              className: ["ox-github-icon"],
              viewBox: "0 0 16 16",
              fill: "currentColor",
            },
            children: [
              {
                type: "element",
                tagName: "path",
                properties: {
                  d: "M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z",
                },
                children: [],
              },
            ],
          },
          {
            type: "element",
            tagName: "span",
            properties: { className: ["ox-github-repo"] },
            children: [{ type: "text", value: repo }],
          },
        ],
      },
    ],
  };
}

function inferLanguage(path: string): string | null {
  const fileName = path.split("/").at(-1)?.toLowerCase() ?? "";
  if (fileName === "dockerfile") return "dockerfile";
  if (fileName === "makefile") return "makefile";

  const extension = fileName.includes(".") ? fileName.split(".").at(-1) : undefined;
  return extension ? (EXTENSION_LANGUAGE_MAP.get(extension) ?? extension) : null;
}

function normalizeSourceLines(content: string): string[] {
  const lines = content.replace(/\r\n?/g, "\n").split("\n");
  if (lines.length > 1 && lines.at(-1) === "") {
    lines.pop();
  }
  return lines.length > 0 ? lines : [""];
}

function createGitHubSourceCard(
  source: GitHubSourceData,
  lines: GitHubLineRange | undefined,
  options: Required<GitHubOptions>,
): Element {
  const allLines = normalizeSourceLines(source.content);
  const start = Math.min(lines?.start ?? 1, allLines.length);
  const end = lines
    ? Math.min(lines.end, allLines.length)
    : Math.min(allLines.length, options.maxSourceLines);
  const selectedLines = allLines.slice(start - 1, end);
  const lineRange = { start, end };
  const loc = selectedLines.length;
  const rangeLabel = formatLineRange(lineRange);
  const locLabel =
    !lines && end < allLines.length
      ? `${rangeLabel} of ${allLines.length} LOC`
      : `${rangeLabel} - ${loc} LOC`;
  const languageClass = source.language ? [`language-${source.language}`] : [];

  return {
    type: "element",
    tagName: "figure",
    properties: {
      className: ["ox-github-code"],
      "data-loc": String(loc),
      "data-source": source.permalink,
    },
    children: [
      {
        type: "element",
        tagName: "figcaption",
        properties: { className: ["ox-github-code-header"] },
        children: [
          {
            type: "element",
            tagName: "a",
            properties: {
              className: ["ox-github-code-title"],
              href: source.permalink,
              target: "_blank",
              rel: "noopener noreferrer",
            },
            children: [{ type: "text", value: `${source.repo}/${source.path}` }],
          },
          {
            type: "element",
            tagName: "span",
            properties: { className: ["ox-github-code-loc"] },
            children: [{ type: "text", value: locLabel }],
          },
        ],
      },
      {
        type: "element",
        tagName: "pre",
        properties: {
          className: ["ox-github-code-block", ...languageClass],
          ...(source.language ? { "data-language": source.language } : {}),
        },
        children: [
          {
            type: "element",
            tagName: "code",
            properties: {
              className: languageClass,
            },
            children: selectedLines.map((line, index) => {
              const lineNumber = start + index;
              return {
                type: "element" as const,
                tagName: "span",
                properties: {
                  className: ["line", "ox-github-code-line"],
                  "data-line": String(lineNumber),
                },
                children: [
                  {
                    type: "element" as const,
                    tagName: "span",
                    properties: { className: ["ox-github-code-line-number"] },
                    children: [{ type: "text" as const, value: String(lineNumber) }],
                  },
                  {
                    type: "element" as const,
                    tagName: "span",
                    properties: { className: ["ox-github-code-line-content"] },
                    children: [{ type: "text" as const, value: line || " " }],
                  },
                ],
              };
            }),
          },
        ],
      },
    ],
  };
}

/**
 * Collect all GitHub repos from HTML for pre-fetching.
 */
export async function collectGitHubRepos(html: string): Promise<string[]> {
  const repos: string[] = [];

  GITHUB_COMPONENT_RE.lastIndex = 0;
  let match;
  while ((match = GITHUB_COMPONENT_RE.exec(html)) !== null) {
    const attrs = parseAttributes(match[1]);
    if (attrs.path || attrs.file || attrs.permalink || attrs.url || attrs.href) {
      continue;
    }

    const repo = attrs.repo;
    if (repo && isSafeGitHubRepo(repo)) {
      repos.push(repo);
    }
  }

  return repos;
}

/**
 * Collect all GitHub source references from HTML for pre-fetching.
 */
export async function collectGitHubSources(html: string): Promise<GitHubSourceRef[]> {
  const sources: GitHubSourceRef[] = [];

  GITHUB_COMPONENT_RE.lastIndex = 0;
  let match;
  while ((match = GITHUB_COMPONENT_RE.exec(html)) !== null) {
    const source = sourceRefFromAttributes(parseAttributes(match[1]));
    if (source) {
      sources.push(source);
    }
  }

  return sources;
}

function parseAttributes(raw: string): Record<string, string> {
  const attrs: Record<string, string> = {};
  ATTRIBUTE_RE.lastIndex = 0;
  let match;

  while ((match = ATTRIBUTE_RE.exec(raw)) !== null) {
    attrs[match[1].toLowerCase()] = match[2] ?? match[3] ?? match[4] ?? "";
  }

  return attrs;
}

function attributesFromElement(el: Element): Record<string, string> {
  const attrs: Record<string, string> = {};
  for (const name of [
    "permalink",
    "url",
    "href",
    "repo",
    "path",
    "file",
    "ref",
    "sha",
    "branch",
    "loc",
    "lines",
    "line",
  ]) {
    const value = getAttribute(el, name);
    if (value !== undefined) {
      attrs[name] = value;
    }
  }
  return attrs;
}

function sourceRefFromAttributes(attrs: Record<string, string>): GitHubSourceRef | null {
  const permalink = attrs.permalink ?? attrs.url ?? attrs.href;
  if (permalink) {
    return parseGitHubPermalink(permalink);
  }

  const repo = attrs.repo;
  const path = attrs.path ?? attrs.file;
  if (!repo || !path || !isSafeGitHubRepo(repo) || !isSafeGitHubPath(path)) {
    return null;
  }

  const ref = attrs.ref ?? attrs.sha ?? attrs.branch ?? "main";
  if (!isSafeGitHubRef(ref)) {
    return null;
  }

  const lines = parseGitHubLineRange(attrs.loc ?? attrs.lines ?? attrs.line);
  const source = { repo, ref, path, lines };
  return {
    ...source,
    permalink: createGitHubPermalink(source),
  };
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

          if (child.type === "element") {
            // Check for <GitHub> component
            if (child.tagName.toLowerCase() === "github") {
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
                const cardElement = repoData
                  ? createGitHubCard(repoData)
                  : createFallbackCard(repo);
                node.children[i] = cardElement;
              }
            } else {
              visit(child);
            }
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
  // If no pre-fetched data, collect and fetch
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
