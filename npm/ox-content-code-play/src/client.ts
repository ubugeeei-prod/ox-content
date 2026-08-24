import { resolveLanguage } from "./catalog";
import {
  resolveCodePlayOptions,
  type RawCodePlayOptions,
  type ResolvedCodePlayOptions,
} from "./config";
import { CodePlaySession } from "./session";
import { createFetchTransport, createUnavailableTransport } from "./transport";
import type { CodePlayTransport, SessionInput, TypeScriptLike } from "./types";

export interface CodePlayClientOptions extends RawCodePlayOptions {
  transport?: CodePlayTransport;
  loadTypeScript?: () => Promise<TypeScriptLike | undefined>;
}

export interface CodePlayClient {
  readonly options: ResolvedCodePlayOptions;
  createSession(input: SessionInput): CodePlaySession;
  hasLanguage(language: string): boolean;
}

export function createCodePlay(options: CodePlayClientOptions = {}): CodePlayClient {
  const resolved = resolveCodePlayOptions(options);
  const transport = options.transport ?? defaultTransport();
  return {
    options: resolved,
    hasLanguage(language: string) {
      const definition = resolveLanguage(language);
      return Boolean(definition && resolved.languages.has(definition.id));
    },
    createSession(input: SessionInput) {
      const definition = resolveLanguage(input.language);
      if (!definition) {
        throw new Error(`Unknown Code Play language: ${input.language}.`);
      }
      const enabled = resolved.languages.get(definition.id);
      if (!enabled) {
        throw new Error(
          `${definition.name} is not enabled. Pass languages.${definition.id}: true to createCodePlay().`,
        );
      }
      return new CodePlaySession({
        ...input,
        definition,
        enabled,
        timeoutMs: resolved.timeoutMs,
        transport,
        endpoints: resolved.endpoints,
        loadTypeScript: options.loadTypeScript,
      });
    },
  };
}

function defaultTransport(): CodePlayTransport {
  if (typeof fetch === "function") {
    return createFetchTransport();
  }
  return createUnavailableTransport();
}
