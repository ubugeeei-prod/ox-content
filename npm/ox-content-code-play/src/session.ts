import { executeAdapter, typecheckAdapter } from "./adapters";
import { mergeConfig } from "./config";
import { errorMessage, errorResult } from "./result";
import { withStdioText } from "./stdio";
import type {
  AdapterRequest,
  CodePlayTransport,
  PlaygroundEndpoints,
  ResolvedLanguageEnable,
  RunResult,
  SessionEventMap,
  SessionEventName,
  SessionInput,
  TypeScriptLike,
} from "./types";
import type { LanguageDefinition } from "./types";

type Listener<T> = (value: T) => void;

export class CodePlaySession {
  readonly language: LanguageDefinition;
  code: string;
  config: Record<string, unknown>;
  lastResult: RunResult | undefined;

  /** Last run's concatenated stdout (same as `lastResult.stdout`). */
  get stdout(): string {
    return this.lastResult?.stdout ?? "";
  }

  /** Last run's concatenated stderr (same as `lastResult.stderr`). */
  get stderr(): string {
    return this.lastResult?.stderr ?? "";
  }

  private readonly enabled: ResolvedLanguageEnable;
  private readonly timeoutMs: number;
  private readonly transport: CodePlayTransport;
  private readonly endpoints: PlaygroundEndpoints;
  private readonly loadTypeScript?: () => Promise<TypeScriptLike | undefined>;
  private readonly listeners = new Map<SessionEventName, Set<Listener<unknown>>>();

  constructor(input: SessionConstructorInput) {
    this.language = input.definition;
    this.enabled = input.enabled;
    this.code = input.code;
    this.config = mergeConfig(input.definition.id, input.enabled, input.config);
    this.timeoutMs = input.timeoutMs;
    this.transport = input.transport;
    this.endpoints = input.endpoints;
    this.loadTypeScript = input.loadTypeScript;
  }

  on<K extends SessionEventName>(event: K, listener: Listener<SessionEventMap[K]>): () => void {
    const bucket = this.listeners.get(event) ?? new Set();
    bucket.add(listener as Listener<unknown>);
    this.listeners.set(event, bucket);
    return () => bucket.delete(listener as Listener<unknown>);
  }

  setCode(code: string): void {
    this.code = code;
  }

  setConfig(config: Record<string, unknown>): void {
    this.config = { ...this.config, ...config };
    this.emit("config", this.config);
  }

  async run(): Promise<RunResult> {
    return this.dispatch("execute");
  }

  async typecheck(): Promise<RunResult> {
    return this.dispatch("typecheck");
  }

  private async dispatch(action: "execute" | "typecheck"): Promise<RunResult> {
    const request: AdapterRequest = {
      definition: this.language,
      enabled: this.enabled,
      code: this.code,
      config: this.config,
      timeoutMs: this.timeoutMs,
      transport: this.transport,
      loadTypeScript: this.loadTypeScript,
      endpoints: this.endpoints,
    };
    try {
      const result = withStdioText(
        action === "typecheck" ? await typecheckAdapter(request) : await executeAdapter(request),
      );
      return this.finish(result);
    } catch (error) {
      return this.finish(errorResult(errorMessage(error)));
    }
  }

  private finish(result: RunResult): RunResult {
    this.lastResult = result;
    for (const event of result.stdio) {
      this.emit("stdio", event);
    }
    this.emit("result", result);
    return result;
  }

  private emit<K extends SessionEventName>(event: K, value: SessionEventMap[K]): void {
    for (const listener of this.listeners.get(event) ?? []) {
      listener(value);
    }
  }
}

export interface SessionConstructorInput extends SessionInput {
  definition: LanguageDefinition;
  enabled: ResolvedLanguageEnable;
  timeoutMs: number;
  transport: CodePlayTransport;
  endpoints: PlaygroundEndpoints;
  loadTypeScript?: () => Promise<TypeScriptLike | undefined>;
}
