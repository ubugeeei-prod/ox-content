import type { OxContentOptions, ResolvedOptions } from "./types";

export function resolveIncludeOptions(
  options: OxContentOptions["includes"],
): ResolvedOptions["includes"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: true, rootDir: options.rootDir };
}
