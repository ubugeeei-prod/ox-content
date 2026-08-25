/**
 * Opt-in git contributor list helpers.
 *
 * Resolution and ignore filtering live here. `git log` is read in Rust
 * (`getGitContributors`) and names are rendered from `PageData.contributors`.
 */

import { createHash } from "node:crypto";
import type { ContributorsOptions, ResolvedContributors } from "./types";

export interface GitContributor {
  name: string;
  email?: string | null;
  commits?: number | null;
}

export interface SsgContributor {
  name: string;
  avatar?: string;
}

/**
 * Resolves `ssg.contributors` with defaults.
 *
 * `false` / omitted stays off. `true` enables names only.
 * An object enables the feature and keeps `ignore` / `avatars`.
 */
export function resolveContributorsOption(
  value: boolean | ContributorsOptions | undefined,
): ResolvedContributors {
  if (!value) {
    return false;
  }
  if (value === true) {
    return { ignore: [], avatars: false };
  }
  return {
    ignore: Array.isArray(value.ignore)
      ? value.ignore.filter((entry): entry is string => typeof entry === "string")
      : [],
    avatars: value.avatars === true,
  };
}

export function filterGitContributors(
  contributors: GitContributor[],
  ignore: string[],
): GitContributor[] {
  if (ignore.length === 0) {
    return contributors.filter((contributor) => contributor.name.trim());
  }
  const needles = new Set(ignore.map((entry) => entry.toLowerCase()));
  return contributors.filter((contributor) => {
    const name = contributor.name.trim();
    if (!name) {
      return false;
    }
    if (needles.has(name.toLowerCase())) {
      return false;
    }
    const email = contributor.email?.trim().toLowerCase();
    return !email || !needles.has(email);
  });
}

export function gravatarAvatar(email: string): string {
  const hash = createHash("md5").update(email.trim().toLowerCase()).digest("hex");
  return `https://www.gravatar.com/avatar/${hash}?d=mp&s=40`;
}

export function applyContributorOptions(
  raw: GitContributor[],
  option: Exclude<ResolvedContributors, false>,
): SsgContributor[] {
  return filterGitContributors(raw, option.ignore).map((contributor) => ({
    name: contributor.name.trim(),
    avatar:
      option.avatars && contributor.email?.trim() ? gravatarAvatar(contributor.email) : undefined,
  }));
}
