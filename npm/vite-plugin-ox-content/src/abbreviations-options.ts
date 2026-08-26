import type { OxContentOptions, ResolvedAbbreviationsOptions } from "./types";

export function resolveAbbreviationsOptions(
  options: OxContentOptions["abbreviations"],
): ResolvedAbbreviationsOptions {
  if (!options) return { enabled: false, terms: {}, firstUseOnly: false };
  if (options === true) return { enabled: true, terms: {}, firstUseOnly: false };
  return {
    enabled: options.enabled ?? true,
    terms: options.terms ?? {},
    firstUseOnly: options.firstUseOnly ?? false,
  };
}
