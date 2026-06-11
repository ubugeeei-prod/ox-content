import type { Element } from "hast";
import { createGitHubPermalink, parseGitHubLineRange, parseGitHubPermalink } from "./source";
import type { GitHubSourceRef } from "./types";
import { isSafeGitHubPath, isSafeGitHubRef, isSafeGitHubRepo } from "./validation";

const GITHUB_COMPONENT_RE = /<github\b([^>]*)>/gi;
const ATTRIBUTE_RE = /([:\w-]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>/]+)))?/g;

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

export function attributesFromElement(el: Element): Record<string, string> {
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

function getAttribute(el: Element, name: string): string | undefined {
  const value = el.properties?.[name];
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return value.join(" ");
  return undefined;
}

export function sourceRefFromAttributes(attrs: Record<string, string>): GitHubSourceRef | null {
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
