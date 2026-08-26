import type { OxContentOptions, ResolvedOptions } from "./types";

export function resolveDefinitionListOptions(
  options: OxContentOptions["definitionLists"],
): NonNullable<ResolvedOptions["definitionLists"]> {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: options.enabled ?? true };
}
