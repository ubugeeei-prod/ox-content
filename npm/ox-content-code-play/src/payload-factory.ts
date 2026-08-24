import { resolveLanguage } from "./catalog";
import type { ResolvedCodePlayOptions } from "./config";
import type { PlayFence } from "./markdown";
import type { PlayPayload } from "./types";

export function payloadFromFence(fence: PlayFence, options: ResolvedCodePlayOptions): PlayPayload {
  const definition = resolveLanguage(fence.language);
  const enabled = definition ? options.languages.get(definition.id) : undefined;
  return {
    language: definition?.id ?? fence.language,
    code: fence.code,
    capabilities: {
      execute: enabled?.execute ?? Boolean(definition?.capabilities.execute),
      typecheck: fence.typecheck || (enabled?.typecheck ?? false),
    },
    config: { ...(definition?.defaultConfig ?? {}), ...(enabled?.config ?? {}) },
    viewers: options.viewers,
    ui: options.ui,
    timeoutMs: options.timeoutMs,
    endpoints: options.endpoints,
  };
}
