/**
 * Shared Code Play types for the headless API, adapters, and UI.
 */

export type StdioStream = "stdin" | "stdout" | "stderr";

export interface StdioEvent {
  stream: StdioStream;
  text: string;
  timestampMs: number;
}

export interface RuntimeLocation {
  /** Hostname, playground origin, or `local`. */
  host: string;
  /** Compiler or runtime name (`tsc`, `rustc`, `node:vm`, …). */
  runtime: string;
  version?: string;
  sandbox?: string;
  target?: string;
}

export interface Provenance {
  compile?: RuntimeLocation;
  execute?: RuntimeLocation;
}

export interface TimingPhase {
  id: string;
  label: string;
  startMs: number;
  durationMs: number;
}

export interface TimingReport {
  totalMs: number;
  phases: TimingPhase[];
}

export type RunStatus = "ok" | "error" | "timeout" | "cancelled" | "unsupported";

export interface Diagnostic {
  message: string;
  severity: "error" | "warning" | "info";
  line?: number;
  column?: number;
  source?: string;
}

export interface PreviewDocument {
  kind: "html";
  html: string;
}

export interface AdapterResult {
  status: RunStatus;
  stdio: StdioEvent[];
  diagnostics: Diagnostic[];
  provenance: Provenance;
  timing: TimingReport;
  preview?: PreviewDocument;
  value?: string;
}

export interface RunResult extends AdapterResult {
  /** Concatenated `stdout` chunks from `stdio`. */
  stdout: string;
  /** Concatenated `stderr` chunks from `stdio`. */
  stderr: string;
}

export type ConfigFieldType = "string" | "boolean" | "select" | "number";

export interface ConfigFieldOption {
  value: string;
  label: string;
}

export interface ConfigField {
  key: string;
  label: string;
  type: ConfigFieldType;
  options?: ConfigFieldOption[];
  default?: unknown;
  description?: string;
}

export type LanguageBackend =
  | "javascript"
  | "typescript"
  | "framework"
  | "rust-playground"
  | "go-playground"
  | "remote";

export type FrameworkId = "vue" | "react" | "svelte" | "solid";

export interface LanguageCapabilities {
  execute: boolean;
  typecheck: boolean;
}

export interface RemoteLanguageSpec {
  pistonLanguage: string;
  pistonVersion?: string;
  notes?: string;
}

export interface LanguageDefinition {
  id: string;
  name: string;
  aliases: string[];
  capabilities: LanguageCapabilities;
  defaultConfig: Record<string, unknown>;
  configSchema: ConfigField[];
  backend: LanguageBackend;
  remote?: RemoteLanguageSpec;
  framework?: FrameworkId;
}

export interface LanguageEnableOptions {
  execute?: boolean;
  typecheck?: boolean;
  endpoint?: string;
  config?: Record<string, unknown>;
}

export type LanguageEnable = boolean | LanguageEnableOptions;

export interface ResolvedLanguageEnable {
  id: string;
  execute: boolean;
  typecheck: boolean;
  endpoint?: string;
  config: Record<string, unknown>;
}

export interface TransportRequest {
  url: string;
  method: "GET" | "POST";
  headers?: Record<string, string>;
  body?: string;
  signal?: AbortSignal;
}

export interface TransportResponse {
  ok: boolean;
  status: number;
  text: string;
}

export interface CodePlayTransport {
  request(input: TransportRequest): Promise<TransportResponse>;
}

export interface ViewerFlags {
  config: boolean;
  stdio: boolean;
  stderr: boolean;
  provenance: boolean;
  timing: boolean;
}

export type CodePlayPreset = "default" | "compact" | "headless";

export interface SessionInput {
  language: string;
  code: string;
  config?: Record<string, unknown>;
}

export interface PlayPayload {
  language: string;
  code: string;
  capabilities: LanguageCapabilities;
  config: Record<string, unknown>;
  viewers: ViewerFlags;
  ui: CodePlayPreset;
  timeoutMs: number;
  endpoints?: PlaygroundEndpoints;
}

export interface TypeScriptLike {
  transpileModule: (
    input: string,
    options: {
      compilerOptions?: Record<string, unknown>;
      reportDiagnostics?: boolean;
      fileName?: string;
    },
  ) => {
    outputText: string;
    diagnostics?: Array<{
      messageText: string | { messageText: string };
      start?: number;
      category: number;
    }>;
  };
  createSourceFile: (
    fileName: string,
    code: string,
    languageVersion: number,
    setParentNodes?: boolean,
  ) => unknown;
  createProgram: (
    rootNames: string[],
    options: Record<string, unknown>,
    host: unknown,
  ) => {
    getSyntacticDiagnostics: () => unknown[];
    getSemanticDiagnostics: () => unknown[];
  };
  flattenDiagnosticMessageText: (message: unknown, newLine: string) => string;
  getLineAndCharacterOfPosition: (
    file: unknown,
    position: number,
  ) => { line: number; character: number };
  ScriptTarget: { Latest: number; ES2022: number };
  ModuleKind: { ESNext: number };
}

export interface AdapterRequest {
  definition: LanguageDefinition;
  enabled: ResolvedLanguageEnable;
  code: string;
  config: Record<string, unknown>;
  timeoutMs: number;
  transport: CodePlayTransport;
  loadTypeScript?: () => Promise<TypeScriptLike | undefined>;
  endpoints: PlaygroundEndpoints;
  signal?: AbortSignal;
}

export interface PlaygroundEndpoints {
  rust: string;
  go: string;
  typecheck?: string;
}

export type SessionEventMap = {
  stdio: StdioEvent;
  result: RunResult;
  config: Record<string, unknown>;
};

export type SessionEventName = keyof SessionEventMap;
