import type { MagicLinkAlias, OxContentOptions, ResolvedMagicLinkOptions } from "./types";

export function resolveMagicLinkOptions(
  options: OxContentOptions["magicLinks"],
): ResolvedMagicLinkOptions {
  if (!options) {
    return { enabled: false, aliases: {}, favicon: false, imageOverrides: [] };
  }
  if (options === true) {
    return { enabled: true, aliases: {}, favicon: false, imageOverrides: [] };
  }
  const favicon =
    options.favicon === true || (typeof options.favicon === "object" && options.favicon != null);
  const faviconTemplate =
    typeof options.favicon === "object" ? options.favicon.template : undefined;
  return {
    enabled: options.enabled ?? true,
    aliases: normalizeAliases(options.aliases),
    favicon,
    faviconTemplate,
    imageOverrides: options.imageOverrides ?? [],
  };
}

function normalizeAliases(
  aliases: Record<string, string | MagicLinkAlias> | undefined,
): Record<string, MagicLinkAlias> {
  const normalized: Record<string, MagicLinkAlias> = {};
  for (const [key, value] of Object.entries(aliases ?? {})) {
    normalized[key] = typeof value === "string" ? { href: value } : value;
  }
  return normalized;
}
