import { resolveLanguage } from "./catalog";
import type { ResolvedCodePlayOptions } from "./config";
import type { PlayFence } from "./markdown";
import { projectSandboxFromPayloadInput } from "./project-sandbox";
import type {
  LanguageDefinition,
  PlayPayload,
  PlaygroundEndpoints,
  ProjectSandboxFile,
} from "./types";

export interface PayloadFromFenceContext {
  files?: ProjectSandboxFile[];
  warnings?: string[];
}

export function payloadFromFence(
  fence: PlayFence,
  options: ResolvedCodePlayOptions,
  context: PayloadFromFenceContext = {},
): PlayPayload {
  const definition = resolveLanguage(fence.language);
  const enabled = definition ? options.languages.get(definition.id) : undefined;
  const payload: PlayPayload = {
    language: definition?.id ?? fence.language,
    code: fence.code,
    title: fence.title,
    capabilities: {
      execute: enabled?.execute ?? Boolean(definition?.capabilities.execute),
      typecheck: payloadTypecheckEnabled(
        fence.typecheck,
        enabled?.typecheck,
        definition,
        options.endpoints,
      ),
    },
    config: { ...definition?.defaultConfig, ...enabled?.config, ...fence.config },
    viewers: { ...options.viewers, ...fence.viewers },
    ui: fence.ui ?? options.ui,
    timeoutMs: fence.timeoutMs ?? options.timeoutMs,
    endpoints: options.endpoints,
  };
  if (enabled?.endpoint) {
    payload.endpoint = enabled.endpoint;
  }
  const project = projectSandboxFromPayloadInput({
    language: payload.language,
    code: payload.code,
    definition,
    project: fence.project,
    files: context.files,
    warnings: context.warnings,
  });
  if (project) {
    payload.project = project;
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
