const GITHUB_REPO_RE = /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/;

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

export function isSafeGitHubRef(ref: string): boolean {
  return Boolean(ref) && !hasControlChar(ref) && !hasUnsafePathSegment(ref);
}

export function isSafeGitHubPath(path: string): boolean {
  return Boolean(path) && !hasControlChar(path) && !hasUnsafePathSegment(path);
}

function hasUnsafePathSegment(value: string): boolean {
  return value
    .split("/")
    .some((part) => !part || part === "." || part === ".." || part.includes("\\"));
}

export function encodePath(path: string): string {
  return path.split("/").map(encodeURIComponent).join("/");
}
