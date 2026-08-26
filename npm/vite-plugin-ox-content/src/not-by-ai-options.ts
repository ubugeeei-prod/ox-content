import type { OxContentOptions, ResolvedOptions } from "./types";

const DEFAULT_NOT_BY_AI_LABEL = "Written by human, not by AI";
const DEFAULT_NOT_BY_AI_HREF = "https://notbyai.fyi";

export function resolveNotByAiOptions(
  options: OxContentOptions["notByAi"],
): ResolvedOptions["notByAi"] {
  if (!options) {
    return { enabled: false, label: DEFAULT_NOT_BY_AI_LABEL, href: DEFAULT_NOT_BY_AI_HREF };
  }
  if (options === true) {
    return { enabled: true, label: DEFAULT_NOT_BY_AI_LABEL, href: DEFAULT_NOT_BY_AI_HREF };
  }
  return {
    enabled: options.enabled ?? true,
    label: options.label?.trim() || DEFAULT_NOT_BY_AI_LABEL,
    href: options.href?.trim() || DEFAULT_NOT_BY_AI_HREF,
  };
}
