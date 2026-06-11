import type { GitHubLineRange, GitHubSourceRef } from "./types";
import { encodePath, isSafeGitHubPath, isSafeGitHubRef, isSafeGitHubRepo } from "./validation";

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

export function sourceKey(source: GitHubSourceRef): string {
  return `${source.repo}@${source.ref}:${source.path}`;
}

export function formatLineRange(lines: GitHubLineRange): string {
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

export function inferLanguage(path: string): string | null {
  const fileName = path.split("/").at(-1)?.toLowerCase() ?? "";
  if (fileName === "dockerfile") return "dockerfile";
  if (fileName === "makefile") return "makefile";

  const extension = fileName.includes(".") ? fileName.split(".").at(-1) : undefined;
  return extension ? (EXTENSION_LANGUAGE_MAP.get(extension) ?? extension) : null;
}
