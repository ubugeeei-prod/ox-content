import type { ParsedProjectOptions } from "./authoring";
import type {
  LanguageDefinition,
  ProjectSandbox,
  ProjectSandboxAdapter,
  ProjectSandboxFile,
  ProjectSandboxProvider,
  ProjectSandboxTarget,
} from "./types";

export interface ProjectSandboxPayloadInput {
  language: string;
  code: string;
  definition?: LanguageDefinition;
  project?: ParsedProjectOptions;
  files?: ProjectSandboxFile[];
  warnings?: string[];
}

export const PROJECT_SANDBOX_ADAPTERS: Record<ProjectSandboxProvider, ProjectSandboxAdapter> = {
  stackblitz: adapter("stackblitz", "StackBlitz", "browser"),
  codesandbox: adapter("codesandbox", "CodeSandbox", "browser"),
  webcontainer: adapter("webcontainer", "WebContainer", "node"),
  external: adapter("external", "External sandbox", "external"),
};

export function projectSandboxFromPayloadInput(
  input: ProjectSandboxPayloadInput,
): ProjectSandbox | undefined {
  if (!input.project) {
    return undefined;
  }
  const warnings = [...(input.warnings ?? [])];
  const adapter = resolveProjectSandboxAdapter(input.project.provider, warnings);
  const sourceFile = normalizeProjectPath(input.project.file, warnings, "source file");
  const entry = normalizeProjectPath(input.project.entry, warnings, "entry path");
  const primary = {
    path: sourceFile ?? entry ?? defaultProjectFile(input.language, input.definition),
    code: input.code,
  };
  const files = mergeProjectFiles([primary, ...(input.files ?? [])]);
  const openUrl = safeProjectUrl(input.project.openUrl, adapter.provider, warnings);
  const fallbackUrl = safeProjectUrl(input.project.fallbackUrl, adapter.provider, warnings);
  return adapter.resolve({
    provider: input.project.provider,
    entry: entry ?? primary.path,
    files,
    openUrl,
    fallbackUrl,
    warnings,
  });
}

export function normalizeProjectPath(
  value: string | undefined,
  warnings: string[] = [],
  label = "file path",
): string | undefined {
  const trimmed = value?.trim();
  if (!trimmed) {
    return undefined;
  }
  const normalized = trimmed.replace(/^\.\//, "");
  if (
    normalized.startsWith("/") ||
    normalized.includes("\\") ||
    normalized.includes("\0") ||
    normalized.split("/").some((part) => part === "" || part === "." || part === "..") ||
    /^[a-z]+:/i.test(normalized)
  ) {
    warnings.push(`Skipped unsafe project ${label}: ${trimmed}`);
    return undefined;
  }
  if (normalized.length > 256) {
    warnings.push(`Skipped long project ${label}: ${trimmed}`);
    return undefined;
  }
  return normalized;
}

export function projectSandboxProviderLabel(provider: ProjectSandboxProvider): string {
  return PROJECT_SANDBOX_ADAPTERS[provider].label;
}

function adapter(
  provider: ProjectSandboxProvider,
  label: string,
  target: ProjectSandboxTarget,
): ProjectSandboxAdapter {
  return {
    provider,
    label,
    target,
    resolve(input) {
      return compactProjectSandbox({
        provider,
        label,
        target,
        entry: input.entry,
        files: input.files,
        openUrl: input.openUrl,
        fallbackUrl: input.fallbackUrl,
        warnings: input.warnings,
      });
    },
  };
}

function defaultProjectFile(language: string, definition: LanguageDefinition | undefined): string {
  if (definition?.framework === "vue") {
    return "src/App.vue";
  }
  if (definition?.framework === "react" || definition?.framework === "solid") {
    return "src/App.tsx";
  }
  if (definition?.framework === "svelte") {
    return "src/App.svelte";
  }
  switch (language) {
    case "javascript":
      return "index.js";
    case "typescript":
      return "index.ts";
    case "go":
      return "main.go";
    case "rust":
      return "src/main.rs";
    default:
      return "snippet.txt";
  }
}

function resolveProjectSandboxAdapter(
  rawProvider: string,
  warnings: string[],
): ProjectSandboxAdapter {
  const provider = rawProvider.trim().toLowerCase();
  switch (provider) {
    case "stackblitz":
    case "stack-blitz":
    case "sb":
      return PROJECT_SANDBOX_ADAPTERS.stackblitz;
    case "codesandbox":
    case "code-sandbox":
    case "csb":
      return PROJECT_SANDBOX_ADAPTERS.codesandbox;
    case "webcontainer":
    case "web-container":
      return PROJECT_SANDBOX_ADAPTERS.webcontainer;
    case "external":
    case "":
      return PROJECT_SANDBOX_ADAPTERS.external;
    default:
      warnings.push(`Unknown project sandbox provider: ${rawProvider}`);
      return PROJECT_SANDBOX_ADAPTERS.external;
  }
}

function mergeProjectFiles(files: ProjectSandboxFile[]): ProjectSandboxFile[] {
  const merged = new Map<string, ProjectSandboxFile>();
  for (const file of files) {
    merged.set(file.path, file);
  }
  return [...merged.values()];
}

function safeProjectUrl(
  value: string | undefined,
  provider: ProjectSandboxProvider,
  warnings: string[],
): string | undefined {
  if (!value) {
    return undefined;
  }
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    warnings.push(`Skipped invalid project URL: ${value}`);
    return undefined;
  }
  if (url.protocol !== "https:" && url.protocol !== "http:") {
    warnings.push(`Skipped non-http project URL: ${value}`);
    return undefined;
  }
  if (url.username || url.password) {
    warnings.push(`Skipped project URL with credentials: ${url.origin}`);
    return undefined;
  }
  if (!providerAllowsHost(provider, url.hostname)) {
    warnings.push(`Skipped ${provider} project URL on unexpected host: ${url.hostname}`);
    return undefined;
  }
  return url.href;
}

function providerAllowsHost(provider: ProjectSandboxProvider, host: string): boolean {
  if (provider === "external") {
    return true;
  }
  const normalized = host.toLowerCase();
  const allowed: Record<Exclude<ProjectSandboxProvider, "external">, string[]> = {
    stackblitz: ["stackblitz.com"],
    codesandbox: ["codesandbox.io", "csb.app"],
    webcontainer: ["webcontainers.io", "webcontainer.io"],
  };
  return allowed[provider].some(
    (suffix) => normalized === suffix || normalized.endsWith(`.${suffix}`),
  );
}

function compactProjectSandbox(project: ProjectSandbox): ProjectSandbox {
  const next: ProjectSandbox = {
    provider: project.provider,
    label: project.label,
    target: project.target,
    files: project.files,
  };
  if (project.entry) {
    next.entry = project.entry;
  }
  if (project.openUrl) {
    next.openUrl = project.openUrl;
  }
  if (project.fallbackUrl) {
    next.fallbackUrl = project.fallbackUrl;
  }
  if (project.warnings?.length) {
    next.warnings = project.warnings;
  }
  return next;
}
