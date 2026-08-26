import { resolveLanguage } from "./catalog";
import type { ResolvedCodePlayOptions } from "./config";
import type { PlayFence } from "./markdown";
import type { LanguageDefinition, PlayPayload, PlaygroundEndpoints } from "./types";

export function payloadFromFence(fence: PlayFence, options: ResolvedCodePlayOptions): PlayPayload {
  const definition = resolveLanguage(fence.language);
  const enabled = definition ? options.languages.get(definition.id) : undefined;
  const payload: PlayPayload = {
    language: definition?.id ?? fence.language,
    code: fence.code,
    capabilities: {
      execute: enabled?.execute ?? Boolean(definition?.capabilities.execute),
      typecheck: payloadTypecheckEnabled(
        fence.typecheck,
        enabled?.typecheck,
        definition,
        options.endpoints,
      ),
    },
    config: { ...definition?.defaultConfig, ...enabled?.config },
    viewers: options.viewers,
    ui: options.ui,
    timeoutMs: options.timeoutMs,
    endpoints: options.endpoints,
  };
  if (enabled?.endpoint) {
    payload.endpoint = enabled.endpoint;
  }
  return payload;
}

/** TypeScript typecheck in the browser needs a reachable endpoint; hide the dead button otherwise. */
export function payloadTypecheckEnabled(
  fenceTypecheck: boolean,
  enabledTypecheck: boolean | undefined,
  definition: LanguageDefinition | undefined,
  endpoints: PlaygroundEndpoints,
): boolean {
  if (!(fenceTypecheck || enabledTypecheck)) {
    return false;
  }
  if (definition?.id === "typescript") {
    return Boolean(endpoints.typecheck);
  }
  return true;
}
