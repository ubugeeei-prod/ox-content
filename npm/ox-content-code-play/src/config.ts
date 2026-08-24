import { listLanguages, resolveLanguage } from "./catalog";
import type {
  CodePlayPreset,
  LanguageEnable,
  PlaygroundEndpoints,
  ResolvedLanguageEnable,
  ViewerFlags,
} from "./types";

export interface ResolvedCodePlayOptions {
  languages: Map<string, ResolvedLanguageEnable>;
  timeoutMs: number;
  ui: CodePlayPreset;
  viewers: ViewerFlags;
  endpoints: PlaygroundEndpoints;
  srcDir?: string;
  outDir?: string;
  base: string;
  proxy: boolean;
}

export interface RawCodePlayOptions {
  languages?: Record<string, LanguageEnable>;
  timeoutMs?: number;
  ui?: CodePlayPreset;
  viewers?: Partial<ViewerFlags>;
  endpoints?: Partial<PlaygroundEndpoints>;
  srcDir?: string;
  outDir?: string;
  base?: string;
  proxy?: boolean;
}

export const DEFAULT_ENDPOINTS: PlaygroundEndpoints = {
  rust: "https://play.rust-lang.org/execute",
  go: "https://play.golang.org/compile",
};

export const DEFAULT_VIEWERS: ViewerFlags = {
  config: true,
  stdio: true,
  stderr: true,
  provenance: true,
  timing: true,
};

export function resolveCodePlayOptions(options: RawCodePlayOptions = {}): ResolvedCodePlayOptions {
  return {
    languages: resolveEnabledLanguages(options.languages ?? {}),
    timeoutMs: options.timeoutMs ?? 10_000,
    ui: options.ui ?? "default",
    viewers: { ...DEFAULT_VIEWERS, ...options.viewers },
    endpoints: { ...DEFAULT_ENDPOINTS, ...options.endpoints },
    srcDir: options.srcDir,
    outDir: options.outDir,
    base: normalizeBase(options.base ?? "/"),
    proxy: options.proxy ?? true,
  };
}

export function resolveEnabledLanguages(
  languages: Record<string, LanguageEnable>,
): Map<string, ResolvedLanguageEnable> {
  const resolved = new Map<string, ResolvedLanguageEnable>();
  for (const [key, enable] of Object.entries(languages)) {
    if (enable === false) {
      continue;
    }
    const definition = resolveLanguage(key);
    if (!definition) {
      throw new Error(`Unknown Code Play language: ${key}.`);
    }
    const explicit = enable === true ? {} : enable;
    resolved.set(definition.id, {
      id: definition.id,
      execute: explicit.execute ?? definition.capabilities.execute,
      typecheck: explicit.typecheck ?? definition.capabilities.typecheck,
      endpoint: explicit.endpoint,
      config: { ...definition.defaultConfig, ...explicit.config },
    });
  }
  return resolved;
}

export function mergeConfig(
  languageId: string,
  enabled: ResolvedLanguageEnable | undefined,
  override?: Record<string, unknown>,
): Record<string, unknown> {
  const definition = resolveLanguage(languageId);
  return {
    ...(definition?.defaultConfig ?? {}),
    ...(enabled?.config ?? {}),
    ...override,
  };
}

export function enabledLanguageIds(): string[] {
  return listLanguages().map((language) => language.id);
}

function normalizeBase(base: string): string {
  if (base === "/" || base === "") {
    return "/";
  }
  return base.endsWith("/") ? base : `${base}/`;
}
