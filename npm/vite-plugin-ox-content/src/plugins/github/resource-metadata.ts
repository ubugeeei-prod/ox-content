import type { GitHubResourceData, GitHubResourceRef } from "./types";

const SHA_RE = /^[A-Fa-f0-9]{7,64}$/;
const GIST_ID_RE = /^[A-Fa-f0-9]{20,64}$/;

interface WorkItemApiData {
  title?: string;
  state?: string;
  html_url?: string;
  body?: string | null;
  comments?: number;
  created_at?: string;
  updated_at?: string;
  user?: { login?: string; avatar_url?: string };
  labels?: Array<string | { name?: string }>;
}

interface CommitApiData {
  sha?: string;
  html_url?: string;
  commit?: {
    message?: string;
    author?: { name?: string; date?: string };
    committer?: { date?: string };
  };
  author?: { login?: string; avatar_url?: string };
}

interface GistApiData {
  id?: string;
  html_url?: string;
  description?: string | null;
  comments?: number;
  created_at?: string;
  updated_at?: string;
  owner?: { login?: string; avatar_url?: string };
  files?: Record<string, { filename?: string }>;
}

export function gitHubResourceDataFromJson(
  resource: GitHubResourceRef,
  value: unknown,
): GitHubResourceData | null {
  const item = record(value);
  if (!item) return null;
  if (resource.kind === "commit") return commitData(resource, item as CommitApiData);
  if (resource.kind === "gist") return gistData(resource, item as GistApiData);
  return workItemData(resource, item as WorkItemApiData);
}

function workItemData(
  resource: GitHubResourceRef,
  item: WorkItemApiData,
): GitHubResourceData | null {
  const title = trimmed(item.title);
  if (!title) return null;
  const dateTime = trimmed(item.updated_at) ?? trimmed(item.created_at);
  return compactData({
    kind: resource.kind,
    repo: resource.repo,
    number: resource.number,
    permalink: resource.permalink,
    title,
    html_url: trimmed(item.html_url) ?? resource.permalink,
    state: trimmed(item.state),
    author: trimmed(item.user?.login),
    avatar_url: safeHttps(trimmed(item.user?.avatar_url)),
    body: excerpt(trimmed(item.body)),
    labels: labels(item.labels),
    comments: count(item.comments),
    dateTime,
    dateLabel: dateLabel(dateTime),
  });
}

function commitData(resource: GitHubResourceRef, item: CommitApiData): GitHubResourceData | null {
  const sha = safeSha(item.sha) ?? resource.sha;
  const message = trimmed(item.commit?.message);
  if (!sha || !message) return null;
  const title = message.split(/\r?\n/, 1)[0]?.replace(/\s+/g, " ").trim() ?? sha.slice(0, 7);
  const dateTime = trimmed(item.commit?.committer?.date) ?? trimmed(item.commit?.author?.date);
  return compactData({
    kind: "commit",
    repo: resource.repo,
    sha,
    permalink: resource.permalink,
    title,
    html_url: trimmed(item.html_url) ?? resource.permalink,
    author: trimmed(item.author?.login) ?? trimmed(item.commit?.author?.name),
    avatar_url: safeHttps(trimmed(item.author?.avatar_url)),
    body: excerpt(message.split(/\r?\n/).slice(1).join("\n")),
    dateTime,
    dateLabel: dateLabel(dateTime),
  });
}

function gistData(resource: GitHubResourceRef, item: GistApiData): GitHubResourceData | null {
  const id = safeGistId(item.id) ?? resource.gistId;
  if (!id) return null;
  const files = fileSummary(item.files);
  const title = trimmed(item.description) ?? (files?.length ? files.join(", ") : `Gist ${id}`);
  const dateTime = trimmed(item.updated_at) ?? trimmed(item.created_at);
  return compactData({
    kind: "gist",
    gistId: id,
    permalink: resource.permalink,
    title,
    html_url: trimmed(item.html_url) ?? resource.permalink,
    author: trimmed(item.owner?.login) ?? resource.gistOwner,
    avatar_url: safeHttps(trimmed(item.owner?.avatar_url)),
    comments: count(item.comments),
    dateTime,
    dateLabel: dateLabel(dateTime),
    files,
  });
}

function labels(value: WorkItemApiData["labels"]): string[] | undefined {
  if (!Array.isArray(value)) return undefined;
  const names = value
    .map((label) => (typeof label === "string" ? label : label.name))
    .map(trimmed)
    .filter((name): name is string => Boolean(name));
  return names.length ? names.slice(0, 8) : undefined;
}

function fileSummary(files: GistApiData["files"]): string[] | undefined {
  if (!files) return undefined;
  const names = Object.values(files)
    .map((file) => trimmed(file.filename))
    .filter(Boolean);
  return names.length ? (names as string[]).slice(0, 4) : undefined;
}

function count(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) && value >= 0
    ? Math.floor(value)
    : undefined;
}

function dateLabel(value: string | undefined): string | undefined {
  return value && /^\d{4}-\d{2}-\d{2}/.test(value) ? value.slice(0, 10) : undefined;
}

function excerpt(value: string | undefined): string | undefined {
  const normalized = value
    ?.replace(/```[\s\S]*?```/g, " ")
    .replace(/`([^`]*)`/g, "$1")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, " ")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/^#+\s*/gm, "")
    .replace(/\s+/g, " ")
    .trim();
  if (!normalized) return undefined;
  return normalized.length <= 220 ? normalized : `${normalized.slice(0, 219).trimEnd()}...`;
}

function safeHttps(value: string | undefined): string | undefined {
  try {
    const url = value ? new URL(value) : null;
    return url?.protocol === "https:" && !url.username && !url.password ? value : undefined;
  } catch {
    return undefined;
  }
}

function safeSha(value: string | undefined): string | undefined {
  return value && SHA_RE.test(value) ? value : undefined;
}

function safeGistId(value: string | undefined): string | undefined {
  return value && GIST_ID_RE.test(value) ? value : undefined;
}

function trimmed(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function record(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}

function compactData(data: GitHubResourceData): GitHubResourceData {
  return Object.fromEntries(
    Object.entries(data).filter(([, value]) => {
      if (Array.isArray(value)) return value.length > 0;
      return value !== undefined && value !== "";
    }),
  ) as GitHubResourceData;
}
