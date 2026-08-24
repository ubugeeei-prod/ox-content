import type { OxContentOptions, ResolvedOptions } from "./types";

export function resolveFileTreeOptions(
  options: OxContentOptions["fileTree"],
): ResolvedOptions["fileTree"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: options.enabled ?? true };
}
