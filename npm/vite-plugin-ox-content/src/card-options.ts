import type { OxContentOptions, ResolvedOptions } from "./types";

export function resolveCardOptions(options: OxContentOptions["cards"]): ResolvedOptions["cards"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: options.enabled ?? true };
}
