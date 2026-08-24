import type { OxContentOptions, ResolvedOptions } from "./types";

export function resolveImageOptions(
  options: OxContentOptions["images"],
): ResolvedOptions["images"] {
  if (!options) return { enabled: false, lazy: true };
  if (options === true) return { enabled: true, lazy: true };
  return { enabled: true, lazy: options.lazy ?? true };
}
