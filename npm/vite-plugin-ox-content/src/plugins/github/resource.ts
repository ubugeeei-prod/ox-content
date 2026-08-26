import type { GitHubResourceKind, GitHubResourceRef } from "./types";
import { isSafeGitHubRepo } from "./validation";

const SHA_RE = /^[A-Fa-f0-9]{7,64}$/;
const GIST_ID_RE = /^[A-Fa-f0-9]{20,64}$/;
const OWNER_RE = /^[A-Za-z0-9_.-]+$/;

export function resourceKey(resource: GitHubResourceRef): string {
  if (resource.kind === "gist") {
    return `gist:${resource.gistId}`;
  }
  if (resource.kind === "commit") {
    return `${resource.repo}:commit:${resource.sha}`;
  }
  return `${resource.repo}:${resource.kind}:${resource.number}`;
}

export function resourceKindLabel(kind: GitHubResourceKind): string {
  switch (kind) {
    case "issue":
      return "issue";
    case "pull":
      return "pull request";
    case "commit":
      return "commit";
    case "discussion":
      return "discussion";
    case "gist":
      return "gist";
  }
}

export function parseGitHubResourceReference(value: string): GitHubResourceRef | null {
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    return null;
  }

  if (url.protocol !== "https:" || url.username || url.password) {
    return null;
  }

  const hostname = url.hostname.toLowerCase();
  const segments = safeSegments(url);
  if (!segments) return null;

  if (hostname === "gist.github.com") {
    return parseGistUrl(segments);
  }

  if (hostname !== "github.com" || segments.length < 4) {
    return null;
  }

  const repo = `${segments[0]}/${segments[1]}`;
  if (!isSafeGitHubRepo(repo)) return null;

  const kind = segments[2];
  if (kind === "issues") {
    return numberedResource("issue", repo, segments[3]);
  }
  if (kind === "pull") {
    return numberedResource("pull", repo, segments[3]);
  }
  if (kind === "discussions") {
    return numberedResource("discussion", repo, segments[3]);
  }
  if (kind === "commit" || kind === "commits") {
    const sha = safeSha(segments[3]);
    if (!sha) return null;
    return {
      kind: "commit",
      repo,
      sha,
      permalink: `https://github.com/${repo}/commit/${sha}`,
      apiUrl: `https://api.github.com/repos/${repo}/commits/${sha}`,
    };
  }

  return null;
}

export function resourceRefFromAttributes(attrs: Record<string, string>): GitHubResourceRef | null {
  const permalink = attrs.permalink ?? attrs.url ?? attrs.href;
  if (permalink) {
    return parseGitHubResourceReference(permalink);
  }

  const repo = attrs.repo;
  if (repo && isSafeGitHubRepo(repo)) {
    const issue = numberedResource("issue", repo, attrs.issue);
    if (issue) return issue;
    const pull = numberedResource("pull", repo, attrs.pull ?? attrs.pr);
    if (pull) return pull;
    const discussion = numberedResource("discussion", repo, attrs.discussion);
    if (discussion) return discussion;
    const sha = safeSha(attrs.commit ?? attrs.sha);
    if (sha) {
      return {
        kind: "commit",
        repo,
        sha,
        permalink: `https://github.com/${repo}/commit/${sha}`,
        apiUrl: `https://api.github.com/repos/${repo}/commits/${sha}`,
      };
    }
  }

  const gist = attrs.gist ?? attrs.gistid;
  if (!gist) return null;
  const gistId = safeGistId(gist);
  if (!gistId) return null;
  const owner = safeOwner(attrs.owner ?? attrs.user);
  return gistResource(gistId, owner);
}

function numberedResource(
  kind: "issue" | "pull" | "discussion",
  repo: string,
  raw: string | undefined,
): GitHubResourceRef | null {
  const number = safeNumber(raw);
  if (!number) return null;
  const pathKind = kind === "pull" ? "pull" : `${kind}s`;
  const apiKind = kind === "discussion" ? "discussions" : "issues";
  return {
    kind,
    repo,
    number,
    permalink: `https://github.com/${repo}/${pathKind}/${number}`,
    apiUrl: `https://api.github.com/repos/${repo}/${apiKind}/${number}`,
  };
}

function parseGistUrl(segments: string[]): GitHubResourceRef | null {
  const [first, second] = segments;
  const owner = second ? safeOwner(first) : undefined;
  const gistId = safeGistId(second ?? first);
  return gistId ? gistResource(gistId, owner) : null;
}

function gistResource(gistId: string, owner: string | undefined): GitHubResourceRef {
  return {
    kind: "gist",
    gistId,
    ...(owner ? { gistOwner: owner } : {}),
    permalink: `https://gist.github.com/${owner ? `${owner}/` : ""}${gistId}`,
    apiUrl: `https://api.github.com/gists/${gistId}`,
  };
}

function safeSegments(url: URL): string[] | null {
  try {
    return url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  } catch {
    return null;
  }
}

function safeNumber(value: string | undefined): number | null {
  if (!value || !/^[1-9]\d{0,8}$/.test(value)) return null;
  const number = Number.parseInt(value, 10);
  return Number.isSafeInteger(number) ? number : null;
}

function safeSha(value: string | undefined): string | undefined {
  return value && SHA_RE.test(value) ? value : undefined;
}

function safeGistId(value: string | undefined): string | undefined {
  return value && GIST_ID_RE.test(value) ? value : undefined;
}

function safeOwner(value: string | undefined): string | undefined {
  return value && OWNER_RE.test(value) && value !== "." && value !== ".." ? value : undefined;
}
