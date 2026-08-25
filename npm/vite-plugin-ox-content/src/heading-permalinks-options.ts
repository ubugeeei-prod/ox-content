import type { OxContentOptions, ResolvedOptions } from "./types";

export function resolveHeadingPermalinksOptions(
  options: OxContentOptions["headingPermalinks"],
): ResolvedOptions["headingPermalinks"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: options.enabled ?? true };
}
