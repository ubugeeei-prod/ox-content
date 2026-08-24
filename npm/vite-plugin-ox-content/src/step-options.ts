import type { OxContentOptions, ResolvedOptions } from "./types";

export function resolveStepsOptions(options: OxContentOptions["steps"]): ResolvedOptions["steps"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: options.enabled ?? true };
}
