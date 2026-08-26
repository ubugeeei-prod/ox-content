import type { OxContentOptions, ResolvedCodeGroupOptions } from "./types";

export function resolveCodeGroupOptions(
  options: OxContentOptions["codeGroups"],
): ResolvedCodeGroupOptions {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: options.enabled ?? true };
}
